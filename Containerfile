FROM rust:1-bullseye AS builder

ARG VERSION=0.8.11
ARG TARGETPLATFORM

WORKDIR /app

COPY . .
RUN cargo build --release 

FROM debian:bullseye-slim

ARG VERSION=0.8.11

LABEL org.opencontainers.image.source=https://github.com/tamada/totebag \
      org.opencontainers.image.version=${VERSION} \
      org.opencontainers.image.title=totebag \
      org.opencontainers.image.description="A tool for extracting/archiving files and directories in multiple formats."

RUN adduser --disabled-password --disabled-login --home /app nonroot \
  && mkdir -p /app
COPY --from=builder /app/target/release/totebag /opt/totebag/totebag

WORKDIR /app
USER nonroot

ENTRYPOINT [ "/opt/totebag/totebag" ]
