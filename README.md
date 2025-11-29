# reth

[![bench status](https://github.com/paradigmxyz/reth/actions/workflows/bench.yml/badge.svg)](https://github.com/paradigmxyz/reth/actions/workflows/bench.yml)
[![CI status](https://github.com/paradigmxyz/reth/workflows/unit/badge.svg)][gh-ci]
[![cargo-lint status](https://github.com/paradigmxyz/reth/actions/workflows/lint.yml/badge.svg)][gh-lint]
[![Telegram Chat][tg-badge]][tg-url]

**Modular, contributor-friendly and blazing-fast implementation of the Ethereum protocol based on reth**

![](./assets/reth-prod.png)

## Installation Instructions
**[Please click here for detailed installation instructions including Docker](examples/altius-reth/README.md)**

[gh-ci]: https://github.com/paradigmxyz/reth/actions/workflows/unit.yml
[gh-lint]: https://github.com/paradigmxyz/reth/actions/workflows/lint.yml
[tg-badge]: https://img.shields.io/endpoint?color=neon&logo=telegram&label=chat&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Fparadigm%5Freth

Note that altius-reth utilizes Paradigm's reth framework to bring Altius technology to the reth project.  To that end, we are building on top of reth so all existing reth flags work the same.  Please see the above installation instructions for more details on how to configure altius-reth.

For everything else that reth already provides including reth specific config options, please visit the reth README and configure your altius-reth instance accordingly.

[Paradigm reth README](https://github.com/paradigmxyz/reth/tree/v1.3.12)

## What is Reth?

Altius-reth is a fully functional ethereum node based on Paradigm's reth. It serves as a user-friendly Execution Layer (EL) and is compatible with all Ethereum Consensus Layer (CL) implementations that support the [Engine API](https://github.com/ethereum/execution-apis/tree/a0d03086564ab1838b462befbc083f873dcf0c0f/src/engine). It is originally built and driven forward by [Paradigm](https://paradigm.xyz/), and is licensed under the Business Source License 1.1 (see [BUSINESS_LICENSE](BUSINESS_LICENSE)).

## Goals

As a full Ethereum node, altius-reth allows users to connect to the Ethereum network and interact with the blockchain but utilizes lower level Altius technology and optimizations to speed up database access and as mentioned execute transactions in parallel.  Since Altius tech is in the lower layers of the reth client, we leverage their existing security and efficiencies, as well as being easy to use on consumer hardware.

More concretely, our goals are:

1. **Performance**: Altius-reth aims to be fast, so we used Rust to extend reth to implement our database performance tech as well as parallelizing the existing revm.
2. **Support as many EVM chains as possible**: By leveraging the existing Reth framework, we aspire that anyone that needs help enhancing database performance and TPS/GPS.  We hope that any chain supported by reth will also use our technology.  As mentioned, Ethereum and Optimism are supported out of the box, but potentially other chains like Polygon, BNB Smart Chain, and more. If you're working on any of these projects, please reach out.
3. **Configurability**: As mentioned, by building in the lower levels of reth we are able to keep all the configurability that reth brings to the table as part of altius-reth as well.  This allows us the maximum flexbility in serving node operators who need super fast performance, as well as hobbyists that might need to run on less powerful hardware.

## Status

Altius-reth is production ready and suitable for usage in mission-critical environments such as staking or high-uptime services. We also actively recommend professional node operators to switch to altius-reth specifically to gain efficiencies in database (SSD) and CPU cloud costs.  Maximum performance == Operation savings.

More historical context below:
* We released an alpha version "production-ready" stable altius-reth in June 2025 based on reth v1.3.12.

### Database compatibility

Note that our current database compability mirrors reth's database compatibility.  Please see their documentation for this information.

Coming soon: Note that once SSMT is deployed, since the database by default will be sharded, our sharded database system will not be compatible with any other reth db.  However, you can always migrate back and forth by syncing against altius-reth -> reth or vice versa from reth -> altius-reth.

<!-- TODO
### Contributing
-->

### Building and testing

<!--
When updating this, also update:
- clippy.toml
- Cargo.toml
- .github/workflows/lint.yml
-->

Note, as mentioned, we follow reth for rust versions.  The Minimum Supported Rust Version (MSRV) of this project is [1.85.0](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/).


## Getting Help

- Join the [Telegram](https://t.me/altiuslabs) to get help, or
<!--- Open a [discussion](https://github.com/Altius-Labs/altius-reth/discussions/new) with your question, or -->
- Open an issue with [the bug](https://github.com/Altius-Labs/altius-reth/issues)

## License
This project is built upon an original codebase licensed under the MIT License
(see LICENSE-MIT) and the Apache License (see LICENSE-APACHE). Those licenses
apply to all unmodified portions of the code.

Forked from [paradigmxyz/reth](https://github.com/paradigmxyz/reth/), this
project is a modified version of reth. See LICENSE-MIT and LICENSE-APACHE for
original license terms.

All modifications and new additions made by Altius Labs Limited are licensed
under the Business Source License 1.1 (see BUSINESS_LICENSE), and are subject to
the terms defined therein. These changes are distributed throughout the
repository.

We have indicated modified files with a licensing comment at the top of each file where applicable.

On the Change Date listed in the BUSINESS_LICENSE, all modified and new code
will automatically be made available under the MIT License (or Apache License
depending on where the code originated).

If you are unsure which parts of the code are governed by which license, please
reach out to licensing@altiuslabs.xyz for clarification.
