# totebag

[![Version](https://shields.io/badge/Version-0.8.0-blue)](https://github.com/tamada/totebag/releases/tag/v0.8.0)
[![MIT License](https://shields.io/badge/License-MIT-blue)](https://github.com/tamada/totebag/blob/main/LICENSE)
[![Docker](https://shields.io/badge/Docker-0.8.0-blue?logo=docker)](https://github.com/tamada/totebag/pkgs/container/totebag)

[![build](https://github.com/tamada/totebag/actions/workflows/build.yaml/badge.svg)](https://github.com/tamada/totebag/actions/workflows/build.yaml)
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/tamada/totebag)](https://rust-reportcard.xuri.me/report/github.com/tamada/totebag)
[![Coverage Status](https://coveralls.io/repos/github/tamada/totebag/badge.svg)](https://coveralls.io/github/tamada/totebag)

A tool for extracting/archiving files and directories in multiple formats.

## :speaking_head: Description

There are many archive formats and their tools.
The one big problem with using the tools is that their interfaces are slightly different and it makes us confused.
Then, the `totebag` treats the archive files as the same interface.
The tool can extract archive files and archive files and directories.

## Usage

```sh
A tool for extracting/archiving files and directories in multiple formats.

Usage: totebag [OPTIONS] [ARGUMENTS]...

Arguments:
  [ARGUMENTS]...  List of files or directories to be processed.
                  '-' reads form stdin, and '@<filename>' reads from a file.
                  In archive mode, the resultant archive file name is determined by the following rule.
                      - if output option is specified, use it.
                      - if the first argument is the archive file name, use it.
                      - otherwise, use the default name 'totebag.zip'.
                  The format is determined by the extension of the resultant file name.

Options:
      --to-archive-name-dir          extract files to DEST/ARCHIVE_NAME directory (extract mode).
  -C, --dir <DIR>                    Specify the base directory for archiving or extracting. [default: .]
  -i, --ignore-types <IGNORE_TYPES>  Specify the ignore type. [possible values: default, hidden, git-ignore, git-global, git-exclude, ignore]
  -L, --level <LEVEL>                Specify the compression level. [default: 5] [possible values: 0-9 (none to finest)]
                                     For more details of level of each compression method, see README. [default: 5]
  -n, --no-recursive                 No recursive directory (archive mode).
  -l, --long                         List entries in the archive file with long format.
      --log <LOGLEVEL>               Specify the log level [default: warn] [possible values: error, warn, info, debug, trace]
  -m, --mode <MODE>                  Mode of operation. [default: auto] [possible values: auto archive, extract, list]
  -o, --output <DEST>                Output file in archive mode, or output directory in extraction mode
      --overwrite                    Overwrite existing files.
  -h, --help                         Print help (see more with '--help')
  -V, --version                      Print version
```

#### Supported archive formats:

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

#### Compression level:

|       | Level                                                        |
| ----- | ------------------------------------------------------------ |
| Cab   | 0: None, otherwise: MsZIP; see [CompressionType](https://docs.rs/cab/latest/cab/enum.CompressionType.html). |
| Gzip  | See [Compression](https://docs.rs/flate2/1.0.35/flate2/struct.Compression.html#method.new). |
| Bzip2 | See [Compression](https://docs.rs/bzip2/latest/bzip2/struct.Compression.html#method.new). |
| Xz    | See [XzEncoder](https://docs.rs/xz/latest/xz/write/struct.XzEncoder.html#method.new). |
| Zstd  | Map 0-9 to 1-22, See [Encoder](https://docs.rs/zstd/latest/zstd/stream/write/struct.Encoder.html#method.new). |
| Zip   | 0: No compression, 1-3: Deflate (10, 24, 264), 4-6: Bzip2 (1, 6, 9), 7-9: Zstd (-7, 3, 22); see [FileOptions.](https://docs.rs/zip/2.2.2/zip/write/struct.FileOptions.html#method.compression_level) |
| 7z    | 0-4: LZMA, 5-9: LZMA64 ([SevenZMethod](https://docs.rs/sevenz-rust/latest/sevenz_rust/struct.SevenZMethod.html)) |

## Install

```sh
brew install tamada/tap/totebag
```

## :whale: Docker

```sh
docker run -it --rm -v $PWD:/workdir ghcr.io/tamada/totebag:0.8.0 [OPTIONS] [ARGUMENTS]...
```

- **Working directory**: `/workdir`
- **User**: `nonroot`

## About

### Authors

- Haruaki Tamada ([tamada](https://github.com/tamada/))

### The Logo and the Origin of totebag

The general word, totebag, is a bag for carrying things.
From this, I chose the name of the tool, totebag, as a tool for packing files and directories carelessly.

![logo](docs/assets/logo.jpeg)

This logo was generated by [Bing Image Creator](https://www.bing.com/images/create/e4b880e381a4e381aee3828ae38293e38194e38292e78987e6898be381a7e6bdb0e38199e794b7e381aee6898be3818ce68f8fe3818be3828ce3819fe38388e383bce38388e38390e38383e382b0e381aee58699e79c9f/1-6614ce41dd1c44aeae12e06dec2e8d68?id=W4JmwP3BnK41FZKKFPisSw%3d%3d&view=detailv2&idpp=genimg&thId=OIG3.H3M7RnPEDRZaxzpZJuii&FORM=GCRIDP&ajaxhist=0&ajaxserp=0).

## Related Tools

- [magiclen/xcompress](https://github.com/magiclen/xcompress)
  - XCompress is a free file archiver utility on Linux, providing multi-format archiving to and extracting from ZIP, Z, GZIP, BZIP2, LZ, XZ, LZMA, 7ZIP, TAR, RAR and ZSTD.
- [meuter/arkiv-rs](https://github.com/meuter/arkiv-rs)
  - Thin convenience library to manipulate compressed archive of vairous types through a single interface.

