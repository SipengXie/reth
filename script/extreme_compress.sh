#!/bin/bash

# 极限压缩脚本 - 使用 xz -9 算法达到最高压缩率
# 用法: ./extreme_compress.sh [目录路径] [压缩方法]
# 默认目录: ssa_cache
# 压缩方法: xz (默认), bz2, gz

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认参数
DEFAULT_DIR="ssa_cache"
TARGET_DIR="${1:-$DEFAULT_DIR}"
COMPRESS_METHOD="${2:-xz}"

# 检查目录是否存在
if [ ! -d "$TARGET_DIR" ]; then
    echo -e "${RED}错误: 目录 '$TARGET_DIR' 不存在${NC}"
    exit 1
fi

# 获取目录的绝对路径和基础名称
TARGET_DIR=$(realpath "$TARGET_DIR")
DIR_BASENAME=$(basename "$TARGET_DIR")

echo -e "${BLUE}==================================================${NC}"
echo -e "${BLUE}           极限压缩工具${NC}"
echo -e "${BLUE}==================================================${NC}"
echo -e "${GREEN}目标目录:${NC} $TARGET_DIR"
echo -e "${GREEN}压缩方法:${NC} $COMPRESS_METHOD"
echo ""

# 计算原始大小
echo -e "${YELLOW}正在计算目录大小...${NC}"
ORIGINAL_SIZE_MB=$(du -sm "$TARGET_DIR" | awk '{print $1}')
ORIGINAL_SIZE_GB=$(echo "scale=2; $ORIGINAL_SIZE_MB / 1024" | bc)
echo -e "${GREEN}原始大小:${NC} ${ORIGINAL_SIZE_MB} MB (${ORIGINAL_SIZE_GB} GB)"
echo ""

# 检测CPU核心数
CPU_CORES=$(nproc)
XZ_THREADS=$((CPU_CORES > 1 ? CPU_CORES : 1))

# 根据压缩方法设置参数
case $COMPRESS_METHOD in
    xz)
        COMPRESS_EXT="tar.xz"
        COMPRESS_NAME="XZ (极限压缩)"
        echo -e "${YELLOW}使用 xz -9 极限压缩 (最高压缩率)${NC}"
        echo -e "${YELLOW}多线程模式: ${XZ_THREADS} 线程${NC}"
        ;;
    bz2|bzip2)
        COMPRESS_EXT="tar.bz2"
        COMPRESS_FLAG="j"
        COMPRESS_NAME="BZIP2 (高压缩)"
        echo -e "${YELLOW}使用 bzip2 -9 高压缩 (平衡压缩率和速度)${NC}"
        ;;
    gz|gzip)
        COMPRESS_EXT="tar.gz"
        COMPRESS_FLAG="z"
        COMPRESS_NAME="GZIP (快速压缩)"
        echo -e "${YELLOW}使用 gzip -9 快速压缩 (较快速度)${NC}"
        ;;
    *)
        echo -e "${RED}错误: 不支持的压缩方法 '$COMPRESS_METHOD'${NC}"
        echo -e "${YELLOW}支持的方法: xz, bz2, gz${NC}"
        exit 1
        ;;
esac

OUTPUT_FILE="${DIR_BASENAME}.${COMPRESS_EXT}"

# 检查输出文件是否已存在
if [ -f "$OUTPUT_FILE" ]; then
    echo -e "${YELLOW}警告: 文件 '$OUTPUT_FILE' 已存在${NC}"
    read -p "是否覆盖? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}操作已取消${NC}"
        exit 1
    fi
    rm -f "$OUTPUT_FILE"
fi

echo ""
echo -e "${BLUE}开始压缩...${NC}"

# 检查是否安装了 pv
if ! command -v pv &> /dev/null; then
    echo -e "${YELLOW}警告: 未检测到 pv 工具，将无法显示实时进度${NC}"
    echo -e "${YELLOW}建议安装: sudo apt install pv${NC}"
    echo -e "${YELLOW}提示: 可以打开另一个终端使用以下命令监控:${NC}"
    echo -e "${YELLOW}  watch -n 2 'ls -lh $OUTPUT_FILE; echo; ps aux | grep xz | grep -v grep'${NC}"
    HAS_PV=0
else
    echo -e "${GREEN}使用 pv 显示打包进度 + xz 显示压缩进度${NC}"
    HAS_PV=1
fi

echo ""

# 记录开始时间
START_TIME=$(date +%s)

# 执行压缩
if [ "$COMPRESS_METHOD" = "xz" ]; then
    # XZ 使用多线程和进度显示
    if [ $HAS_PV -eq 1 ]; then
        echo -e "${YELLOW}[tar 打包] → [pv 监控] → [xz 多线程压缩]${NC}"
        echo ""
        tar -cf - -C "$(dirname "$TARGET_DIR")" "$DIR_BASENAME" | \
            pv -p -t -e -r -b | \
            xz -9 -T${XZ_THREADS} -v > "$OUTPUT_FILE"
    else
        tar -cf - -C "$(dirname "$TARGET_DIR")" "$DIR_BASENAME" | \
            xz -9 -T${XZ_THREADS} -v > "$OUTPUT_FILE"
    fi
else
    # 其他压缩方法使用原来的 tar 命令
    tar -c${COMPRESS_FLAG}f "$OUTPUT_FILE" -C "$(dirname "$TARGET_DIR")" "$DIR_BASENAME"
fi

# 检查压缩是否成功
if [ ${PIPESTATUS[0]} -ne 0 ] || [ ! -f "$OUTPUT_FILE" ]; then
    echo -e "${RED}压缩失败!${NC}"
    exit 1
fi

# 记录结束时间
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
MINUTES=$((DURATION / 60))
SECONDS=$((DURATION % 60))

# 获取压缩后的大小
if [ -f "$OUTPUT_FILE" ]; then
    COMPRESSED_SIZE_MB=$(du -sm "$OUTPUT_FILE" | awk '{print $1}')
    COMPRESSED_SIZE_GB=$(echo "scale=2; $COMPRESSED_SIZE_MB / 1024" | bc)

    # 计算压缩率
    COMPRESSION_RATIO=$(echo "scale=2; (($ORIGINAL_SIZE_MB - $COMPRESSED_SIZE_MB) * 100) / $ORIGINAL_SIZE_MB" | bc)
    SIZE_RATIO=$(echo "scale=2; ($COMPRESSED_SIZE_MB * 100) / $ORIGINAL_SIZE_MB" | bc)

    echo ""
    echo -e "${BLUE}==================================================${NC}"
    echo -e "${GREEN}压缩完成!${NC}"
    echo -e "${BLUE}==================================================${NC}"
    echo -e "${GREEN}压缩方法:${NC} $COMPRESS_NAME"
    echo -e "${GREEN}原始大小:${NC} ${ORIGINAL_SIZE_MB} MB (${ORIGINAL_SIZE_GB} GB)"
    echo -e "${GREEN}压缩后:${NC}   ${COMPRESSED_SIZE_MB} MB (${COMPRESSED_SIZE_GB} GB)"
    echo -e "${GREEN}压缩率:${NC}   ${COMPRESSION_RATIO}%"
    echo -e "${GREEN}剩余大小:${NC} ${SIZE_RATIO}% 的原始大小"
    echo -e "${GREEN}耗时:${NC}     ${MINUTES}分${SECONDS}秒"
    echo -e "${GREEN}输出文件:${NC} $OUTPUT_FILE"
    echo -e "${BLUE}==================================================${NC}"
else
    echo -e "${RED}错误: 压缩文件未生成${NC}"
    exit 1
fi
