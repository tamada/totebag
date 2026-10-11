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

ARG GIT_REVISION
ARG BUILD_DATE
ARG VERSION=0.9.0
ARG AUTHOR="Haruaki Tamada"
ARG TITLE=totebag
ARG URL=https://tamada.github.io/totebag
ARG SOURCE=https://github.com/tamada/totebag
ARG LICENSE=MIT

LABEL org.opencontainers.image.authors=${AUTHOR} \
      org.opencontainers.image.title=${TITLE} \
      org.opencontainers.image.description="A tool for extracting/archiving files and directories in multiple formats." \
      org.opencontainers.image.url=${URL} \
      org.opencontainers.image.source=${SOURCE} \
      org.opencontainers.image.version=${VERSION} \
      org.opencontainers.image.revision=${GIT_REVISION} \
      org.opencontainers.image.created=${BUILD_DATE} \
      org.opencontainers.image.licenses=${LICENSE}

RUN useradd --create-home --home-dir /app --shell /usr/sbin/nologin nonroot
COPY --from=builder /app/target/release/totebag /opt/totebag/totebag

WORKDIR /app
USER nonroot

ENTRYPOINT [ "/opt/totebag/totebag" ]
