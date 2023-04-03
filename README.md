# BioMeat - Scrape and download media from MangaDex

[![License](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)

## Overview

BioMeat scrape images from [MangaDex website](https://mangadex.org/).

## Installing

Pre-compiled binaries can be downloaded from the
[Releases](https://github.com/TehUncleDolan/biomeat/releases/) page.

Alternatively, BioMeat can be installed from Cargo, via the following command:

```
cargo install biomeat
```

BioMeat can be built from source using the latest stable or nightly Rust.
This is primarily useful for developing on BioMeat.

```
git clone https://github.com/TehUncleDolan/biomeat.git
cd biomeat
cargo build --release
cp target/release/biomeat /usr/local/bin
```

BioMeat follows Semantic Versioning.

## Usage

```text
Download manga from Mangadex

Usage: biomeat [OPTIONS] --manga <MANGA>

Options:
  -o, --output <OUTPUT>  Path to the output directory [default: .]
  -m, --manga <MANGA>    Manga ID
  -l, --lang <LANG>      Lang [default: en]
  -s, --start <START>    Start downloading from the specified chapter number [default: 0]
  -h, --help             Print help
  -V, --version          Print version
```

The simplest invocation only requires you to specify the UUID of the series you
want to download, the other options have sensible defaults.

```text
biomeat -m d6d1812d-639a-41b8-a304-e21fc9a1e5bc
```
