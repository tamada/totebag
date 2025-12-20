# totebag

[![Version](https://shields.io/badge/Version-0.8.2-blue)](https://github.com/tamada/totebag/releases/tag/v0.8.2)
[![MIT License](https://shields.io/badge/License-MIT-blue)](https://github.com/tamada/totebag/blob/main/LICENSE)
[![Docker](https://shields.io/badge/Docker-0.8.2-blue?logo=docker)](https://github.com/tamada/totebag/pkgs/container/totebag)

[![build](https://github.com/tamada/totebag/actions/workflows/build.yaml/badge.svg)](https://github.com/tamada/totebag/actions/workflows/build.yaml)
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/tamada/totebag)](https://rust-reportcard.xuri.me/report/github.com/tamada/totebag)
[![Coverage Status](https://coveralls.io/repos/github/tamada/totebag/badge.svg)](https://coveralls.io/github/tamada/totebag)

A tool for extracting/archiving files and directories in multiple formats.

## :speaking_head: Description

There are many archive formats and their tools.
The one big problem with using the tools is that their interfaces are slightly different, which makes us confused.
Then the totebag treats the archive files as if they were on the same interface.
The tool can extract archive files and directories.

- The CLI interface is provided in the `cli` directory. See [cli/README.md](cli/README.md) for more details.
- The API is provided in the `totebag` crate. See [docs.rs/totebag](https://docs.rs/totebag), and [lib/README.md](lib/README.md) for more details.

## Supported archive formats

- [Cab](https://github.com/mdsteele/rust-cab)
- [Tar](https://crates.io/crates/tar)
- Tar+[Gzip](https://crates.io/crates/flate2)
- Tar+[Bzip2](https://crates.io/crates/bzip2)
- Tar+[Xz](https://crates.io/keywords/xz)
- Tar+[Zstd](https://crates.io/crates/zstd)
- [Zip](https://crates.io/crates/zip)
- [7z](https://crates.io/crates/sevenz-rust)
- [Lha, Lzh](https://github.com/royaltm/rust-delharc) (extraction only)
- [Rar](https://crates.io/crates/unrar) (extraction only)

## :smile: About

### Authors

- Haruaki Tamada ([tamada](https://github.com/tamada/))

### The Logo and the Origin of totebag

The general term "totebag" refers to a bag for carrying things.
From this, I chose the name `totebag` for the tool, as a way to pack files and directories carelessly.

![logo](docs/assets/logo.jpeg)

[Bing Image Creator](https://www.bing.com/images/create/e4b880e381a4e381aee3828ae38293e38194e38292e78987e6898be381a7e6bdb0e38199e794b7e381aee6898be3818ce68f8fe3818be3828ce3819fe38388e383bce38388e38390e38383e382b0e381aee58699e79c9f/1-6614ce41dd1c44aeae12e06dec2e8d68?id=W4JmwP3BnK41FZKKFPisSw%3d%3d&view=detailv2&idpp=genimg&thId=OIG3.H3M7RnPEDRZaxzpZJuii&FORM=GCRIDP&ajaxhist=0&ajaxserp=0) created this logo.

## Related Tools

- [magiclen/xcompress](https://github.com/magiclen/xcompress)
  - XCompress is a free file archiver utility on Linux, providing multi-format archiving to and extracting from ZIP, Z, GZIP, BZIP2, LZ, XZ, LZMA, 7ZIP, TAR, RAR, and ZSTD.
- [meuter/arkiv-rs](https://github.com/meuter/arkiv-rs)
  - Thin convenience library to manipulate compressed archives of various types through a single interface.

