---
menus: ["main"]
title: "⚓️ Install"
weight: 20
date: 2024-06-11
---

## :beer: Homebrew

```sh
brew install tamada/tap/totebag
```

## Download

Download the suitable binary from the [release page](https://github.com/tamada/totebag/releases/latest).
Then, the unpack the downloaded archive and put the binary to the directory in the `PATH`.

## :whale: Docker

```sh
docker run -it --rm -v $PWD:/workdir ghcr.io/tamada/totebag:$VERSION [OPTIONS] [ARGUMENTS]...
```

- **Working directory**: `/workdir`
- **User**: `nonroot`
