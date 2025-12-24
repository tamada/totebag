## CLI interface for totebag

### :speaking_head: Overview

The cli interface for `totebag` provides a command-line tool to extract and archive files and directories in multiple formats.
It offers a unified interface for various archive formats, making it easier to manage archives without worrying about the differences between tools.

### :runner: Usage

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
  -C, --dir <DIR>                    Specify the base directory for archiving or extracting.
                                     [default: .]
  -i, --ignore-types <IGNORE_TYPES>  Specify the ignore type.
                                     [possible values: default, hidden, git-ignore, git-global, git-exclude, ignore]
  -L, --level <LEVEL>                Specify the compression level. [default: 5]
                                     [possible values: 0-9 (none to finest)]
                                     For more details of level of each compression method, see README. [default: 5]
  -n, --no-recursive                 No recursive directory (archive mode).
  -f, --output-format <FORMAT>       Specify the format for listing entries in the archive file.
                                     [default: default] [possible values: default, long, json, pretty-json, xml]
      --log <LOGLEVEL>               Specify the log level [default: warn]
                                     [possible values: error, warn, info, debug, trace]
  -m, --mode <MODE>                  Mode of operation. [default: auto]
                                     [possible values: auto, archive, extract, list]
  -F, --from <ARCHIVE_FORMAT>        Specify the archive format for listing mode (default auto). 
                                     available on list and extract modes.
                                     [possible values: auto, parse, cab, lha, lzh, seven-z, rar, tar, tar-gz, tar-bz2, tar-xz, tar-zstd, zip, tgz, tbz2, txz, tzst, tzstd, jar, war, ear]
  -o, --output <DEST>                Output file in archive mode, or output directory in extraction mode
      --overwrite                    Overwrite existing files.
  -h, --help                         Print help (see more with '--help')
  -V, --version                      Print version
```

Supported archive formats include Cab, Tar, Tar with Gzip/Bzip2/Xz/Zstd, Zip, 7z, Lha/Lzh (extraction only), and Rar (extraction only).
See [README.md](../README.md) for more details.

#### :whale: Docker

```sh
docker run -it --rm -v $PWD:/workdir ghcr.io/tamada/totebag:latest [OPTIONS] [ARGUMENTS]...
```

- **Working directory**: `/app`
- **User**: `nonroot`

For more detail, see [Containerfile](../Containerfile)

### Detecting the archive format

`totebag` automatically detects the archive format by the extension of the archive file name in the default.
However, you can explicitly specify the archive format by using the `--from` option in the list and extract modes.
If `--from` option is absent or `auto`, which means automatic detection by the extension of the archive file name.
If `--from parse` is specified, the archive format is detected by parsing the magic number of the archive file (read header and detect the format).
Otherwise, the specified format is used.

### Examples

#### List the file names in the archive file

```sh
totebag -m list archive.zip
```

#### Extract files from the archive file

```sh
totebag archive.zip
```

This is the minimum command to extract the given file.
The mode is automatically set to extract mode, since the all of arguments are archive files.
The archive format is determined by the extension of the archive file name.
The above command is equivalent to the following command:

```sh
totebag -m extract -o . archive.zip
```

#### Create an archive file from files and directories

```sh
totebag archive.zip file1 dir1 file2
``` 

This is the minimum command to create an archive file named `archive.zip` including `file1`, `dir1`, and `file2`.
The mode is automatically set to archive mode, since the first argument is the archive file name and the others are not archive files.
The archive format is determined by the extension of the archive file name.
The above command is equivalent to the following command:

```sh
totebag -m archive -o archive.zip file1 dir1 file2
```

### :anchor: Install

```sh
brew install tamada/tap/totebag
```
