VERSION := `grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/g'`

# Format code with rustfmt
format:
    cargo fmt --all

# Check formatting without modifying files (as CI does)
format-check:
    cargo fmt --all --check

# Run clippy with warnings as errors
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build the project (runs format and clippy first)
build: format clippy
    cargo build

# Run tests with coverage
test:
    cargo llvm-cov --html

# Assert the default feature set pulls in no C toolchain dependency
check-pure:
    #!/usr/bin/env bash
    found=$(cargo tree -e normal,build --workspace \
      | grep -oE '(cc|pkg-config|lzma-sys|zstd-sys|bzip2-sys|unrar_sys) v[0-9.]+' \
      | sort -u || true)
    if [ -n "$found" ]; then echo "C dependency found:"; echo "$found"; exit 1; fi
    echo "default build is pure Rust"

# Generate shell completions
generate-completion:
    cargo run -p totebag-cli --features completion -- --generate-completion
    cp -r target/completions ./assets

# Start the document server
start:
    hugo -s docs server

# Build the site document
site:
    hugo -s docs

# Build the docker image for the current arch
docker:
    docker build -t ghcr.io/tamada/totebag:latest -t ghcr.io/tamada/totebag:{{VERSION}} .

# Build the docker image for arm64
docker_arm64:
    docker build --platform linux/arm64/v8 -t ghcr.io/tamada/totebag:latest -t ghcr.io/tamada/totebag:{{VERSION}} .

# Build the docker image for amd64
docker_amd64:
    docker build --platform linux/amd64 -t ghcr.io/tamada/totebag:latest -t ghcr.io/tamada/totebag:{{VERSION}} .

# Build the docker image for multi-arch
docker_buildx:
    docker buildx build --platform linux/arm64/v8,linux/amd64 --output=type=image,push=true -t ghcr.io/tamada/totebag:latest -t ghcr.io/tamada/totebag:{{VERSION}} .
