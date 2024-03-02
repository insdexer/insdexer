# Insdexer

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](#license)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.75%2B-green)](#rust-version-requirements)

A high-performance, highly compatible EVM Inscriptions Indexer by Rust.

An accessible and complete version of the documentation is available at **[insdexer.gitbook.io](https://insdexer.gitbook.io)**.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.

## Rust Version Requirements

1.75

<!--ts-->

* [System Requirements](#system-requirements)
* [Usage](#usage)
  * [Getting Started](#getting-started)
  * [Logging](#logging)
  * [RESTful-API](#restful-api)

* [Key features](#key-features)
  * [Compatible Inscribe](#more-efficient-state-storage)
  * [Efficient Storage](#more-efficient-state-storage)
  * [Multi-Thread Sync](#faster-initial-sync)
  * [JSON-RPC daemon](#json-rpc-daemon)

* [Documentation](#documentation)
* [FAQ](#faq)
* [Getting in touch](#getting-in-touch)
  * [Reporting security issues/concerns](#reporting-security-issues/concerns)
  * [Team](#team)

<!--te-->

**Disclaimer**: this software is currently a tech preview. We will do our best to keep it stable and make no breaking changes but we don't guarantee anything. Things can and will break.

## System Requirements

* CPU: 4-core (or 2-core hyperthreaded).

* RAM: >=16GB, 64-bit architecture.

* Storage: > 500Gb.
    SSD or NVMe. Do not recommend HDD.
    Bear in mind that SSD performance deteriorates when close to capacity.

## Usage

### Getting Started
