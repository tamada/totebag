FROM rust:1-alpine3.20 AS builder

RUN apk --no-cache add musl-dev

WORKDIR /app

COPY Cargo.toml .
RUN    mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release

COPY src    /app/src
COPY assets /app/assets
RUN    touch src/main.rs \
    && cargo build --release \
    && strip target/release/totebag -o totebag

FROM gcr.io/distroless/static-debian12:nonroot
USER nonroot

ARG VERSION=0.7.3

LABEL org.opencontainers.image.authors="Haruaki Tamada <tamada@users.noreply.github.com>" \
    org.opencontainers.image.url="https://github.com/tamada/totebag" \
    org.opencontainers.image.documentation="A tool for extracting/archiving files and directories in several formats." \
    org.opencontainers.image.source="https://github.com/tamada/totebag/blob/main/Dockerfile" \
    org.opencontainers.image.version="${VERSION}"

WORKDIR /app

ENV HOME=/app
ENV BTMEISTER_HOME=/opt/totebag

COPY --from=builder /app/totebag /opt/totebag/totebag

ENTRYPOINT [ "/opt/totebag/totebag" ]
