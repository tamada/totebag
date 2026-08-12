## API of totebag crate

### :speaking_head: Overview

This is the README for the `totebag` crate, which provides the API of the totebag tool for extracting/archiving files and directories in multiple formats.

The `totebag` crate provides a unified API for handling various archive formats, making it easy for developers to integrate archiving and extraction functionality into their Rust applications.
It abstracts the differences between various archive formats, providing a consistent interface for working with archives.

Every default dependency is a pure Rust implementation, so building `totebag` requires no
C toolchain and cross-compiling needs nothing but a Rust target.

#### Supported archive formats

- [Ar](https://crates.io/crates/ar)
- [Cab](https://crates.io/crates/cab)
- [Cpio](https://crates.io/crates/kpea)
- [Tar](https://crates.io/crates/tar)
- Tar+[Gzip](https://crates.io/crates/flate2)
- Tar+[Bzip2](https://crates.io/crates/bzip2)
- Tar+[Xz](https://crates.io/crates/lzma-rust2)
- Tar+[Zstd](https://crates.io/crates/ruzstd)
- [Zip](https://crates.io/crates/zip)
- [7z](https://crates.io/crates/sevenz-rust2)
- [Lha, Lzh](https://crates.io/crates/delharc) (extraction only)
- [Rar](https://crates.io/crates/unrar) (extraction only, behind the `rar` feature)

#### Cargo features

| Feature | Default | Effect |
| ------- | :-----: | ------ |
| `clap` | off | Derives `clap::ValueEnum` for `IgnoreType` and `OutputFormat` so they can be used directly as command line arguments. |
| `rar` | off | Enables RAR extraction. See [RAR support](#rar-support). |
| `zstd-native` | off | Uses the C zstd library instead of `ruzstd`. See [zstd compression levels](#zstd-compression-levels). |

##### RAR support

There is no pure Rust RAR implementation. The `unrar` crate links the C UnRAR library,
whose license forbids using its source to re-create the RAR compression algorithm, so
RAR is not compiled in by default:

```console
$ totebag -m list some.rar
Rar: support is not compiled in (rebuild with --features rar)
```

The released binaries and container images are built with `--features rar`. To get it in
your own build, enable the feature:

```sh
cargo add totebag --features rar
```

##### zstd compression levels

The default zstd backend is [`ruzstd`](https://crates.io/crates/ruzstd), which implements
the whole decompression side of the specification but only the fastest compression level.
`.tar.zst` archives therefore compress less tightly than `zstd(1)` would, and the
`--level` option only distinguishes "store" (0) from "compress" (1-9).

Enable `zstd-native` to link the C library instead and get the full 0-9 range mapped onto
zstd's 1-22.

### :walking: How to use

#### :green_heart: Archiving files and directories

```rust
use std::path::PathBuf;

let config = totebag::ArchiveConfig::builder()
    .dest("results/test.zip")         // destination file.
    .rebase_dir(PathBuf::from("new")) // prefix for every entry in the archive.
    .overwrite(true)                  // set overwrite flag of the destination file.
    .build();
let targets: Vec<PathBuf> = ["src", "Cargo.toml"].iter() // files to be archived.
    .map(PathBuf::from).collect();
match totebag::archive(&targets, &config) {
    Ok(_) => println!("archiving is done"),
    Err(e) => eprintln!("error: {:?}", e),
}
```

##### Entry names

Entry names are normalized by [`normalize_entry_path`] before they reach the format
backend, so an archive `totebag` creates never contains a name that starts at the
filesystem root or climbs out with `..`:

```rust
use std::path::{Path, PathBuf};
use totebag::normalize_entry_path;

assert_eq!(normalize_entry_path(Path::new("../project/src/main.rs")), PathBuf::from("project/src/main.rs"));
assert_eq!(normalize_entry_path(Path::new("/etc/hosts")), PathBuf::from("etc/hosts"));
assert_eq!(normalize_entry_path(Path::new("./src/main.rs")), PathBuf::from("src/main.rs"));
```

`rebase_dir` is applied on top of the normalized path.

[`normalize_entry_path`]: https://docs.rs/totebag/latest/totebag/fn.normalize_entry_path.html

#### :yellow_heart: Extracting the archive file

```rust
let config = totebag::ExtractConfig::builder()
    .dest("results") // set the destination directory.
    .build();
match totebag::extract("extracting_archive_file.zip", &config) {
    Ok(r) => println!("{:?}", r),
    Err(e) => println!("error: {:?}", e),
}
```

##### Compression level

|       | Level                                                        |
| ----- | ------------------------------------------------------------ |
| Ar    | N/A                                                          |
| Cab   | 0: None, otherwise: MsZIP; see [CompressionType](https://docs.rs/cab/latest/cab/enum.CompressionType.html). |
| Cpio  | 0-3: Odc, 4-6: Newc, 7: Crc, 8: Bin(LittleEndian), 9: Bin(BigEndian); see [`kpea::Format`](https://docs.rs/kpea/0.2.5/kpea/enum.Format.html). |
| Gzip  | Passed through as-is; see [Compression](https://docs.rs/flate2/latest/flate2/struct.Compression.html#method.new). |
| Bzip2 | Passed through as-is; see [Compression](https://docs.rs/bzip2/latest/bzip2/struct.Compression.html#method.new). |
| Xz    | Used as the preset; see [`XzOptions::with_preset`](https://docs.rs/lzma-rust2/latest/lzma_rust2/struct.XzOptions.html). |
| Zstd  | 0: stored, 1-9: fastest. With `zstd-native`, mapped onto zstd's 1-22; see [Encoder](https://docs.rs/zstd/latest/zstd/stream/write/struct.Encoder.html#method.new). |
| Zip   | 0: No compression, 1-3: Deflate (10, 24, 264), 4-6: Bzip2 (1, 6, 9), 7-9: Xz (3, 6, 9); see [FileOptions](https://docs.rs/zip/latest/zip/write/struct.FileOptions.html#method.compression_level). |
| 7z    | 0-4: LZMA, 5-9: LZMA2; see [`EncoderMethod`](https://docs.rs/sevenz-rust2/latest/sevenz_rust2/struct.EncoderMethod.html). |

#### :blue_heart: List entries in an archive file

The `list` function returns a string-formatted list of entries in the archive file.

```rust
use totebag::{ListConfig, OutputFormat, format::default_format_detector};

let config = ListConfig::new(OutputFormat::Default, default_format_detector());
match totebag::list("listing_archive_file.zip", &config) {
    Ok(formatted_list) => println!("{}", formatted_list),
    Err(e) => println!("error: {:?}", e),
}
```

The `entries` function returns the entries recorded in the archive file.

```rust
use totebag::format::default_format_detector;

let detector = default_format_detector();
match totebag::entries("listing_archive_file.zip", detector.as_ref()) {
    Ok(entries) => {
        for entry in entries.iter() {
            println!("{}", entry.name);
        }
    }
    Err(e) => println!("error: {:?}", e),
}
```
