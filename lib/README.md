## API of totebag crate

### :speaking_head: Overview

This is the README for the `totebag` crate, which provides the API of the totebag tool for extracting/archiving files and directories in multiple formats.

The `totebag` crate provides a unified API for handling various archive formats, making it easy for developers to integrate archiving and extraction functionality into their Rust applications.
It abstracts the differences between various archive formats, providing a consistent interface for working with archives.

#### Supported archive formats

- [Cab](https://crates.io/crates/cab)
- [Cpio](https://crates.io/crates/kpea)
- [Tar](https://crates.io/crates/tar)
- Tar+[Gzip](https://crates.io/crates/flate2)
- Tar+[Bzip2](https://crates.io/crates/bzip2)
- Tar+[Xz](https://crates.io/keywords/xz)
- Tar+[Zstd](https://crates.io/crates/zstd)
- [Zip](https://crates.io/crates/zip)
- [7z](https://crates.io/crates/sevenz-rust)
- [Lha, Lzh](https://crates.io/crates/delharc) (extraction only)
- [Rar](https://crates.io/crates/unrar) (extraction only)

### :walking: How to use

#### :green_heart: Archiving files and directories

```rust
use std::path::PathBuf;

let config = totebag::ArchiveConfig::builder()
    .dest("results/test.zip")         // destination file.
    .rebase_dir(PathBuf::from("new")) // rebased directory in the archive file.
    .overwrite(true)                  // set overwrite flag of the destination file.
    .build();
let targets: Vec<PathBuf> = vec!["src", "Cargo.toml"].iter() // files to be archived.
    .map(|s| PathBuf::from(s)).collect::<Vec<_>>();   
match totebag::archive(&targets, &config) {
    Ok(_) => println!("archiving is done"),
    Err(e) => eprintln!("error: {:?}", e),
}
```

#### :yellow_heart: Extracting the archive file

```rust
use std::path::PathBuf;

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
| Cab   | 0: None, otherwise: MsZIP; see [CompressionType](https://docs.rs/cab/latest/cab/enum.CompressionType.html). |
| Gzip  | See [Compression](https://docs.rs/flate2/1.0.35/flate2/struct.Compression.html#method.new). |
| Bzip2 | See [Compression](https://docs.rs/bzip2/latest/bzip2/struct.Compression.html#method.new). |
| Xz    | See [XzEncoder](https://docs.rs/xz/latest/xz/write/struct.XzEncoder.html#method.new). |
| Zstd  | Map 0-9 to 1-22, See [Encoder](https://docs.rs/zstd/latest/zstd/stream/write/struct.Encoder.html#method.new). |
| Zip   | 0: No compression, 1-3: Deflate (10, 24, 264), 4-6: Bzip2 (1, 6, 9), 7-9: Zstd (-7, 3, 22); see [FileOptions.](https://docs.rs/zip/2.2.2/zip/write/struct.FileOptions.html#method.compression_level) |
| 7z    | 0-4: LZMA, 5-9: LZMA64 ([SevenZMethod](https://docs.rs/sevenz-rust/latest/sevenz_rust/struct.SevenZMethod.html)) |

#### :blue_heart: List entries in an archive file

The `list` function returns a string-formatted list of entries in the archive file.

```rust
use std::path::PathBuf;

let file = PathBuf::from("listing_archive_file.zip");
let config = totebag::ListConfig::new(totebag::OutputFormat::Default);
match totebag::list(file, &config) {
    Ok(formatted_list) => println!("{:?}", formatted_list),
    Err(e) => println!("error: {:?}", e),
}
```

The `entries` function returns a vector of Entry objects that are reflected in the archive file.

```rust
use std::path::PathBuf;

let file = PathBuf::from("listing_archive_file.zip");
match totebag::entries(file) {
    Ok(entries) => println!("{:?}", entries),
    Err(e) => println!("error: {:?}", e),
}
```
