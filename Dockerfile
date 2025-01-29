FROM rust:1-bullseye AS builder

ARG VERSION=0.7.4
ARG TARGETPLATFORM

WORKDIR /work/totebag

COPY . .
RUN cargo build --release 

FROM debian:bullseye-slim

ARG VERSION=0.7.4

LABEL org.opencontainers.image.source=https://github.com/tamada/totebag \
      org.opencontainers.image.version=${VERSION} \
      org.opencontainers.image.title=totebag \
      org.opencontainers.image.description="totebag is a simple file transfer tool."

RUN adduser --disabled-password --disabled-login --home /workdir nonroot \
  && mkdir -p /workdir
COPY --from=builder /work/totebag/target/release/totebag /opt/totebag/totebag

WORKDIR /workdir
USER nonroot

ENTRYPOINT [ "/opt/totebag/totebag" ]
