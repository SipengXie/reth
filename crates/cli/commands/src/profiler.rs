use reth_node_core::version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    default, fs,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};
use sysinfo::System;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
#[derive(Serialize, Deserialize)]
pub struct TraceMonitor {
    out_dir: PathBuf,
    ssa_enabled: bool,
    parallel_enabled: bool,
    prewarm_enabled: bool,
    cli_version: String,
    timestamp: String,
    is_release: bool,
    hardware: String,
    #[serde(skip)]
    chrome_guard: Arc<tokio::sync::Mutex<Option<tracing_chrome::FlushGuard>>>,
}

struct BlockData {
    block_num: Option<String>,
    data: Vec<String>,
}

struct TracingWriter {
    sender: mpsc::Sender<BlockData>,
    system_info: String,
    buffer: Vec<String>,
    inside_block: bool,
    current_block: Option<String>,
    partial_buf: String, // Cache incomplete JSON
}

impl TracingWriter {
    fn new(sender: mpsc::Sender<BlockData>, system_info: String) -> Self {
        Self {
            sender,
            system_info,
            buffer: Vec::new(),
            inside_block: false,
            current_block: None,
            partial_buf: String::new(),
        }
    }

    fn process_json(&mut self, value: Value) {
        if let Value::Object(map) = &value {
            let line_str = serde_json::to_string(&value).unwrap_or_default();
            let cat = map.get("cat").and_then(|v| v.as_str()).unwrap_or("");
            let ph = map.get("ph").and_then(|v| v.as_str()).unwrap_or("");

            if cat == "block_profiler" && ph == "B" {
                let now =
                    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
                let millis = now.as_millis();
                let mut system: TraceMonitor = serde_json::from_str(&self.system_info).unwrap();
                system.timestamp = millis.to_string();
                self.inside_block = true;
                self.buffer.clear();
                self.buffer.push(serde_json::to_string(&system).unwrap());
                self.buffer.push(line_str);

                if let Some(block_num) =
                    map.get("args").and_then(|v| v.get("block_num")).and_then(|v| v.as_str())
                {
                    self.current_block = Some(block_num.to_string());
                }
                return;
            }

            if self.inside_block {
                self.buffer.push(line_str);

                if cat == "block_profiler" && ph == "E" {
                    self.inside_block = false;
                    let block_data = BlockData {
                        block_num: self.current_block.clone(),
                        data: self.buffer.clone(),
                    };

                    if let Err(_) = self.sender.try_send(block_data) {
                        eprintln!("Tracing channel is full, dropping block data.");
                    }
                    self.current_block = None;
                }
            }
        }
    }
}

impl Write for TracingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_all(buf)?;
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        // Concatenate data to cache
        let mut chunk = String::from_utf8_lossy(buf).to_string();

        // Remove leading '['
        if chunk.starts_with('[') {
            chunk = chunk[1..].to_string();
        }
        // Remove the end ']'
        if chunk.ends_with(']') {
            chunk = chunk[..chunk.len() - 1].to_string();
        }
        // Concatenate to cache
        self.partial_buf.push_str(&chunk);

        // Split by ",\n"
        let partial_buf_clone = self.partial_buf.clone();
        let mut parts: Vec<&str> = partial_buf_clone.split(",\n").collect();

        // Last part may be incomplete JSON, save it
        self.partial_buf = parts.pop().unwrap_or("").to_string();

        // Parse previous complete JSON
        for part in parts {
            let cleaned = part.trim().trim_matches(',');
            if cleaned.is_empty() {
                continue;
            }
            match serde_json::from_str::<Value>(cleaned) {
                Ok(value) => self.process_json(value),
                Err(e) => eprintln!("Parse failed: {} Original text: {}", e, cleaned),
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Process remaining JSON on flush
        let cleaned = self.partial_buf.trim().trim_matches(',');
        if !cleaned.is_empty() {
            if let Ok(value) = serde_json::from_str::<Value>(cleaned) {
                self.process_json(value);
            }
        }
        self.partial_buf.clear();
        Ok(())
    }
}

impl TraceMonitor {
    pub fn start(&mut self, prewarm: bool) {
        if !self.is_enabled() {
            tracing_subscriber::registry()
                .with(EnvFilter::from_default_env())
                .with(tracing_subscriber::fmt::layer())
                .init();
            return;
        }

        let (sender, receiver) = mpsc::channel(100);
        let system_info = serde_json::to_string(&self).unwrap_or_default();
        let writer = TracingWriter::new(sender, system_info);

        let (chrome_layer, guard) =
            ChromeLayerBuilder::new().writer(writer).include_args(true).build();

        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .with(chrome_layer)
            .init();

        self.chrome_guard = Arc::new(tokio::sync::Mutex::new(Some(guard)));
        self.prewarm_enabled = prewarm;
        self.run(receiver);
    }

    fn run(&self, mut receiver: mpsc::Receiver<BlockData>) {
        if !self.is_enabled() {
            return;
        }
        let out_dir = self.out_dir.clone();
        if out_dir.exists() {
            let _ = fs::remove_dir_all(&out_dir);
        }
        let _ = fs::create_dir_all(&out_dir);
        tokio::spawn(async move {
            while let Some(block_data) = receiver.recv().await {
                let filename = match &block_data.block_num {
                    Some(num) => format!("block_{}.json", num),
                    None => "block_unknown.json".to_string(),
                };
                let filepath: PathBuf = out_dir.join(&filename);
                if let Ok(mut out) = File::create(&filepath).await {
                    let json_array = format!("[\n{}\n]", block_data.data.join(",\n"));
                    if let Err(e) = out.write_all(json_array.as_bytes()).await {
                        eprintln!("Failed to write block file: {:?}", e);
                    }
                } else {
                    eprintln!("Failed to create block file: {:?}", filepath);
                }
            }
        });

        // Chrome flush
        let chrome_guard = Arc::clone(&self.chrome_guard);
        tokio::spawn(async move {
            loop {
                {
                    let guard_opt = chrome_guard.lock().await;
                    if let Some(guard) = guard_opt.as_ref() {
                        let _ = guard.flush();
                    }
                }
                sleep(Duration::from_millis(500)).await;
            }
        });
    }

    fn is_enabled(&self) -> bool {
        env_flag("ENABLE_CHROME_TRACE")
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).map(|v| v.eq_ignore_ascii_case("true") || v == "1").unwrap_or(false)
}

impl default::Default for TraceMonitor {
    fn default() -> Self {
        let version = version::default_client_version();
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_memory = sys.total_memory();
        let cpus = sys.cpus();
        let cpu_brand = cpus.first().map(|c| c.brand()).unwrap_or("unknown");
        Self {
            out_dir: PathBuf::from("block_perfetto"),
            ssa_enabled: env_flag("ENABLE_SSA"),
            parallel_enabled: env_flag("ENABLE_PARALLEL"),
            prewarm_enabled: false,
            cli_version: serde_json::to_string(&version).unwrap_or_default(),
            is_release: cfg!(not(debug_assertions)),
            hardware: format!(
                "os={} arch={} family={} cpu_cores={} total_mem={}MB cpu_brand={}",
                std::env::consts::OS,
                std::env::consts::ARCH,
                std::env::consts::FAMILY,
                cpus.len(),
                total_memory / 1024 / 1024,
                cpu_brand
            ),
            chrome_guard: Arc::new(tokio::sync::Mutex::new(None)),
            timestamp: "".to_string(),
        }
    }
}
