FROM rust:1-trixie AS builder

ARG VERSION=0.9.0
ARG TARGETPLATFORM

WORKDIR /app

# Only what the build needs. COPY puts the *contents* of a source directory into
# the destination, so each directory gets its own line to keep its name.
# rust-toolchain.toml pins the compiler, exactly as in CI.
COPY Cargo.toml rust-toolchain.toml ./
COPY lib ./lib
COPY cli ./cli

RUN cargo build --release

FROM debian:trixie-slim

ARG VERSION=0.9.0

LABEL org.opencontainers.image.source=https://github.com/tamada/totebag \
      org.opencontainers.image.version=${VERSION} \
      org.opencontainers.image.title=totebag \
      org.opencontainers.image.description="A tool for extracting/archiving files and directories in multiple formats."

RUN useradd --create-home --home-dir /app --shell /usr/sbin/nologin nonroot
COPY --from=builder /app/target/release/totebag-cli /opt/totebag/totebag

WORKDIR /app
USER nonroot

ENTRYPOINT [ "/opt/totebag/totebag" ]
