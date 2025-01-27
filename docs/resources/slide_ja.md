---
title: "totebag -- A tool for archiving files and directories and extracting several archive formats."
author: "Haruaki Tamada"
slide: true
marp: true
theme: default
---

# totebag 

## A tool for extracting/archiving files and directories in several formats

![](./docs/assets/logo.jpeg)

Haruaki Tamada (@tamada)
https://github.com/tamada/totebag

---

# Why totebag?

- 世の中には多くの圧縮，解凍ツールがある．
  - しかし，それらを扱うツールの利用方法は統一されていない．
- 各ツールの使い方をいちいち調べるのが面倒だ！
- 一つのツールで，複数の圧縮形式を扱おう！
  - 最近の言語は，ライブラリが充実しているのでできるはず！

---

# 作ってみた．

- **ツール名**　　totebag
- **言語**　　　　Rust
- **ロゴ作成**　　Microsoft image creator（AI）
- **タグライン**
  - A tool for archiving files and directories and extracting several archive formats

---

# サポートしている圧縮形式

- Lha/Lzh（解凍のみ）
- Rar（解凍のみ）
- 7Zip
- Tar（tar, tar+gz, tar+xz, tar+bz2）
- Zip（zip, jar, war, ear）

---

# Usage

```sh
A tool for extracting/archiving files and directories in multiple formats.

Usage: totebag [OPTIONS] [ARGUMENTS]...

Arguments:
  [ARGUMENTS]...  List of files or directories to be processed.

Options:
  -m, --mode <MODE>          Mode of operation. [default: auto]
                             [possible values: auto, archive, extract, list]
  -o, --output <DEST>        Output file in archive mode, or output directory 
                             in extraction mode
      --to-archive-name-dir  extract files to DEST/ARCHIVE_NAME directory
                             (extract mode).
  -n, --no-recursive         No recursive directory (archive mode).
  -v, --verbose              Display verbose output.
      --overwrite            Overwrite existing files.
  -h, --help                 Print help
  -V, --version              Print version
```

---

# 処理内容

- 解凍
  - コマンドライン引数全てが圧縮ファイルだと解凍モードになる．
- リスト
  - `mode` に `list` を指定すると，アーカイブ内のファイル一覧を表示する．
- 圧縮
  - `mode` が `auto` で，コマンドライン引数に圧縮ファイル以外が指定されていると圧縮モードになる．

---

# Rust

- コンパイルがものすごく面倒．
  - コンパイルが通るとメモリ関連の実行時エラーがほぼ起こらない．
  - だから良い，という意見もわかるし，慣れてくると楽になる．
- けれど，実装に取り掛かるのにものすごく気合がいる言語だなぁ．

---

# Cargo

- :+1: 言語標準！
- :+1: パッケージマネージャ機能が付いている．
- :+1: 様々な拡張機能が導入可能．
- :-1: 拡張機能の導入に準備が必要．
- :-1: 外部コマンドを実行できない．
- 短気（プログラマの三大美徳の一つ）が満たされない．
  - 新たに導入しようとするときに，準備のコマンドを何度も入力しないといけない．
  - 環境によって動作する/しないが異なってくる．

---

# 学んだこと

- Rust
  - ある程度思い通りにプログラムを組めるようになった．
    - 2年前に学習したときは，非常に苦労した上に，思い通りにならないことが多かった．
- Marp
  - いつまでもスライド作りを PowerPoint に頼るのは嫌．
    - テキスト形式でスライドを作りたい．
      - 授業資料，論文，プログラムなどほぼ全ての成果物をGitHubで管理しているため．
    - レイアウトが思い通りにならない．．．
    - 後から pptx を編集できない．．．

---

# まとめ

- 複数の圧縮形式を取り扱うツール totebag を作成した．
  - Rust で作成した．
  - Rust の学習につながった．
- Marp を使い続けるのは様子見かな？？？