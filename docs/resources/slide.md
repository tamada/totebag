---
title: "totebag -- A tool for archiving files and directories and extracting several archive formats."
author: "Haruaki Tamada"
slide: true
marp: true
theme: default
---

# totebag 

## A tool for extracting/archiving files and directories in several formats

![](../assets/logo.jpeg)

Haruaki Tamada (@tamada)
https://github.com/tamada/totebag

---

# Why totebag?

- There are many archivers in the world.
  - The one problem is that their usage is slightly different.
- It is bothersome to learn each tool.
- Let us create a tool to support the several archive formats.
  - Today's programming languages provide widespread libraries, including archivers.
  - Hence, it is moderately easy to build.

---

# Build Tool

- **Tool Name**
  - totebag
- **Language**
  - Rust
- **Logo creation**
  - Microsoft image creator（AI）
- **Tagline**
  - A tool for archiving files and directories and extracting several archive formats

---

# Supported archive formats

- Cab
- Lha/Lzh (extraction only)
- Rar (extraction only)
- 7-Zip
- Tar（tar, tar+gz, tar+xz, tar+bz2, tar+zstd）
- Zip（zip, jar, war, ear）

---

# Usage

```sh
A tool for extracting/archiving files and directories in multiple formats.

Usage: totebag [OPTIONS] [ARGUMENTS]...

Arguments:
  [ARGUMENTS]...  List of files or directories to be processed.
                  If archive mode, the archive file name can specify at the first argument.
                  If the frist argument was not the archive name, the default archive name `totebag.zip` is applied.


Options:
      --to-archive-name-dir          extract files to DEST/ARCHIVE_NAME directory (extract mode).
  -C, --dir <DIR>                    Specify the base directory for archiving or extracting. [default: .]
  -i, --ignore-types <IGNORE_TYPES>  Specify the ignore type. [possible values: default, hidden, git-ignore, git-global, git-exclude, ignore]
  -n, --no-recursive                 No recursive directory (archive mode).
  -l, --long                         List entries in the archive file with long format.
      --level <LEVEL>                Specify the log level [default: warn] [possible values: error, warn, info, debug]
  -m, --mode <MODE>                  Mode of operation. [default: auto] [possible values: auto, archive, extract, list]
  -o, --output <DEST>                Output file in archive mode, or output directory in extraction mode
      --overwrite                    Overwrite existing files.
  -h, --help                         Print help
  -V, --version                      Print version
```

---

# How to understand mode

The default mode is `auto`.
In `auto` mode, `totebag` decides the concrete process routine from the given arguments.

- **Extract**
  - If the `mode` is `auto` and the given arguments are all archive files, `totebag` works as an `extract` mode.
- **List**
  - If the `mode` is `list`, `totebag` lists the entries of the archive file given from the arguments.
- **Archive**
  - If the `mode` is `auto` and the given arguments contains non-archive files, `totebag` works as an `archive` mode.

---

# Miscellaneous impression about Rust

- :-1: It is very hard to compile.
  - However, the compiled program rarely causes memory-related runtime errors.
  - The difficulty of compilation will be relieved by the experiences of the programmer.
- The rust requires a lot of effort to get started with the projects.

---

# Cargo

- :+1: Language standard!
- :+1: Package manager!
- :+1: Various extensions are available.
- :-1: Preparation is required to install the extensions.
  - Impatient (one of the three virtues of a programmer) is not satisfied.
    - We must enter the preparation command repeatedly when introducing a new extension in each environment.
    - The behavior depends on the environment.
- :-1: Cannot execute external commands.

---

# What is I learn from the Rust project?

- Rust
  - I have become able to program to as much extent as I want.
  - When I studied two years ago, I had a lot of trouble and often couldn't do what I wanted.
- Marp
  - I don't want to rely on PowerPoint to make presentation materials.
    - I want to make slides in text format.
      - I manage almost all of my work, such as class materials, papers, and programs on GitHub.
    - The layout doesn't turn out as I want it to...
    - I can't edit the pptx afterwards...

---

# Summary

- I created a tool `totebag` that handles multiple compression formats.
  - I created it in Rust.
  - It was a good opportunity to learn Rust.
- I'm not sure if I will continue to use Marp...
