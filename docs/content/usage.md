---
menus: ["main"]
title: "🏃‍♀️ Usage"
weight: 40
date: 2024-06-11
---

```sh
A tool for extracting/archiving files and directories in multiple formats.

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
- Tar+Zstd
- Zip
- 7z
- Lha, Lzh (extraction only)
- Rar (extraction only)