# totebag

[![Version](https://shields.io/badge/Version-0.3.0-blue)](https://github.com/tamada/totebag/releases/tag/v0.3.0)
[![MIT License](https://shields.io/badge/License-MIT-blue)](https://github.com/tamada/totebag/blob/main/LICENSE)

[![build](https://github.com/tamada/totebag/actions/workflows/build.yaml/badge.svg)](https://github.com/tamada/totebag/actions/workflows/build.yaml)
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/tamada/totebag)](https://rust-reportcard.xuri.me/report/github.com/tamada/totebag)
[![Coverage Status](https://coveralls.io/repos/github/tamada/totebag/badge.svg)](https://coveralls.io/github/tamada/totebag)

A tool for archiving files and directories and extracting several archive formats.

## Description

There are many archive formats and their tools. The one problem with using each tool is that its interfaces are slightly different.
Then, The `totebag` treats the archive files as the same interface.
The tool can extract archive files and archive files and directories.

## Usage

```sh
A tool for archiving files and directories and extracting several archive formats.

Usage: totebag [OPTIONS] [ARGUMENTS]...

Arguments:
  [ARGUMENTS]...  List of files or directories to be processed.

Options:
  -m, --mode <MODE>          Mode of operation. [default: auto] [possible values: auto, archive, extract, list]
  -o, --output <DEST>        Output file in archive mode, or output directory in extraction mode
      --to-archive-name-dir  extract files to DEST/ARCHIVE_NAME directory (extract mode).
  -n, --no-recursive         No recursive directory (archive mode).
  -v, --verbose              Display verbose output.
      --overwrite            Overwrite existing files.
  -h, --help                 Print help
  -V, --version              Print version
```

Supported archive formats:

- Tar
- Tar+Gzip
- Tar+Bzip2
- Tar+Xz
- Zip
- 7z
- Rar (extraction only)

## Install

```sh
brew install tamada/tap/totebag
```

## About

### Authors

* Haruaki Tamada ([tamada](https://github.com/tamada/))

### The Logo and the Origin of totebag

The general word, totebag, is a bag for carrying things.
From this, I chose the name of the tool, totebag, as a tool for packing files and directories carelessly.

![logo](site/assets/logo.jpeg)

This logo was generated by [Bing Image Creator](https://www.bing.com/images/create/e4b880e381a4e381aee3828ae38293e38194e38292e78987e6898be381a7e6bdb0e38199e794b7e381aee6898be3818ce68f8fe3818be3828ce3819fe38388e383bce38388e38390e38383e382b0e381aee58699e79c9f/1-6614ce41dd1c44aeae12e06dec2e8d68?id=W4JmwP3BnK41FZKKFPisSw%3d%3d&view=detailv2&idpp=genimg&thId=OIG3.H3M7RnPEDRZaxzpZJuii&FORM=GCRIDP&ajaxhist=0&ajaxserp=0).
