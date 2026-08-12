---
title: "🏃‍♀️ Usage"
weight: 40
date: 2024-06-11
---

```sh
A tool for extracting/archiving files and directories in multiple formats.

Usage: totebag [OPTIONS] [ARGUMENTS]...

Arguments:
  [ARGUMENTS]...  List of files or directories to be processed.
                  '-' reads from stdin, and '@<filename>' reads from a file.
                  In archive mode, the resultant archive file name is determined by the following rule.
                      - if output option is specified, use it.
                      - if the first argument is the archive file name, use it.
                      - otherwise, use the default name 'totebag.zip'.
                  The format is determined by the extension of the resultant file name.

Options:
      --to-archive-name-dir          extract files to DEST/ARCHIVE_NAME directory (extract mode).
  -C, --rebase-dir <DIR>             Prefix every entry in the archive with DIR (archive mode).
                                     For example, -C root stores src/main.rs as root/src/main.rs.
                                     By default entries keep their own paths. [aliases: --dir]
  -i, --ignore-types <IGNORE_TYPES>  Specify the ignore type.
                                     [possible values: default, hidden, git-ignore, git-global, git-exclude, ignore]
  -L, --level <LEVEL>                Specify the compression level. [default: 5]
                                     [possible values: 0-9 (none to finest)]
                                     For more details of level of each compression method, see README.
  -n, --no-recursive                 No recursive directory (archive mode).
  -f, --output-format <FORMAT>       Specify the format for listing entries in the archive file.
                                     [default: default] [possible values: default, long, json, pretty-json, xml]
      --log <LOGLEVEL>               Specify the log level [default: warn]
                                     [possible values: error, warn, info, debug, trace]
  -m, --mode <MODE>                  Mode of operation. [default: auto]
                                     [possible values: auto, archive, extract, list]
  -F, --from <ARCHIVE_FORMAT>        Specify the archive format for listing mode (default auto).
                                     available on list and extract modes.
                                     [possible values: auto, parse, ar, cab, cpio, lha, lzh, seven-z, rar, tar,
                                      tar-gz, tar-bz2, tar-xz, tar-zstd, zip, tgz, tbz2, txz, tzst, tzstd, jar,
                                      war, ear]
  -o, --output <DEST>                Output file in archive mode, or output directory in extraction mode
      --overwrite                    Overwrite existing files.
  -h, --help                         Print help (see more with '--help')
  -V, --version                      Print version
```

## Supported archive formats

| Format | Extensions | Archive | Extract |
| ------ | ---------- | :-----: | :-----: |
| Ar | `.ar`, `.a`, `.lib` | ✅ | ✅ |
| Cab | `.cab` | ✅ | ✅ |
| Cpio | `.cpio` | ✅ | ✅ |
| Lha, Lzh | `.lha`, `.lzh` | — | ✅ |
| 7z | `.7z` | ✅ | ✅ |
| Rar | `.rar` | — | ✅ (needs the `rar` feature) |
| Tar | `.tar` | ✅ | ✅ |
| Tar+Gzip | `.tar.gz`, `.tgz` | ✅ | ✅ |
| Tar+Bzip2 | `.tar.bz2`, `.tbz2` | ✅ | ✅ |
| Tar+Xz | `.tar.xz`, `.txz` | ✅ | ✅ |
| Tar+Zstd | `.tar.zst`, `.tzst`, `.tar.zstd`, `.tzstd` | ✅ | ✅ |
| Zip | `.zip`, `.jar`, `.war`, `.ear` | ✅ | ✅ |

RAR support is not built by default: no pure Rust RAR implementation exists, and the
UnRAR license forbids using its source to re-create the RAR compression algorithm.
The released binaries and container images are built with it enabled; if you install
from crates.io and need it, use `cargo install totebag-cli --features rar`.

## Entry names in the created archive

Entry names are normalized before they are written, so an archive `totebag` creates
never contains a name that starts at the filesystem root or climbs out with `..`:

| target | entry name |
| ------ | ---------- |
| `src/main.rs` | `src/main.rs` |
| `./src/main.rs` | `src/main.rs` |
| `../project/src/main.rs` | `project/src/main.rs` |
| `/etc/hosts` | `etc/hosts` |

This matches what `tar(1)` does when it reports "Removing leading `../' from member
names". Use `-C/--rebase-dir` to put everything under a prefix instead:

```sh
totebag -C myapp-1.0 myapp-1.0.tar.gz src README.md
```

stores `myapp-1.0/src/...` and `myapp-1.0/README.md`.

## Detecting the archive format

`totebag` detects the archive format from the file name extension by default.
Use `--from` to override it in list and extract modes:

- `--from auto` (or omitting it) detects the format by the extension.
- `--from parse` reads the file header and detects the format by its magic number.
- any other value forces that format regardless of the file name.
