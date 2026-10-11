git_revision := `git rev-parse --short HEAD`
app_version := `awk -F'"' '/^\[workspace.package\]/{p=1} p && /^version *=/{print $2; exit}' Cargo.toml`
build_date := `date -u +%Y-%m-%dT%H:%M:%SZ`
container_image := 'quay.io/tama5'

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

# Generate shell completions into assets/completions, replacing what is there.
# clap_complete includes the hidden --generate-completion option; remove it
# from the generated files by hand before committing them.
generate-completion:
    rm -rf target/completions
    cargo run -p totebag-cli --features completion -- --generate-completion
    rm -rf assets/completions
    cp -r target/completions assets/completions

# Start the document server
start:
    hugo -s docs server

# Build the site document
site:
    hugo -s docs

# Build the docker image for the current arch
docker:
    docker build \
         --build-arg GIT_REVISION={{git_revision}} \
         --build-arg BUILD_DATE={{build_date}} \
         --build-arg VERSION={{app_version}} \
         -t {{container_image}}/totebag:latest \
         -t {{container_image}}/totebag:{{ app_version }} \
         -f Containerfile \
         .

# Build the docker image for multi-arch
docker_buildx:
    docker buildx build --push \
         --platform linux/amd64,linux/arm64 \
         --build-arg GIT_REVISION={{git_revision}} \
         --build-arg BUILD_DATE={{build_date}} \
         --build-arg VERSION={{app_version}} \
         -t {{container_image}}/totebag:latest \
         -t {{container_image}}/totebag:{{ app_version }} \
         -f Containerfile \
         .
