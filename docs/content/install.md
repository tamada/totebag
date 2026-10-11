---
title: "⚓️ Install"
weight: 20
date: 2024-06-11
---

## :beer: Homebrew

```sh
brew install tamada/tap/totebag
```

## :crab: Cargo

Install from [crates.io](https://crates.io/crates/totebag-cli) with Rust's package manager:

```sh
cargo install totebag-cli
```

This installs the `totebag` command. Every format, RAR included, is built in, so
no Cargo feature needs to be enabled. Building needs Rust 1.95 or later, but no C
toolchain.

## Download

Download the suitable binary from the [release page](https://github.com/tamada/totebag/releases/latest).
Then, the unpack the downloaded archive and put the binary to the directory in the `PATH`.

## :whale: Docker

```sh
docker run -it --rm -v $PWD:/app quay.io/tama5/totebag:latest [OPTIONS] [ARGUMENTS]...
```

- **Working directory**: `/app`
- **User**: `nonroot`

Mount the directory you want to work on at `/app`, which is the image's working
directory; paths given on the command line are resolved relative to it.
