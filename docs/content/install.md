---
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
docker run -it --rm -v $PWD:/app quay.io/tama5/totebag:latest [OPTIONS] [ARGUMENTS]...
```

- **Working directory**: `/app`
- **User**: `nonroot`

Mount the directory you want to work on at `/app`, which is the image's working
directory; paths given on the command line are resolved relative to it.
