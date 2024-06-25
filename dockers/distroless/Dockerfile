FROM rust:latest AS builder

ARG VERSION=0.4.6
ARG TARGETPLATFORM

WORKDIR /work/totebag

COPY . .
RUN cargo build --release 

FROM gcr.io/distroless/cc

ARG VERSION=0.4.6

LABEL org.opencontainers.image.source https://github.com/tamada/totebag

COPY --from=builder /work/totebag/target/release/totebag /opt/totebag/totebag

WORKDIR /workdir
USER nonroot

ENTRYPOINT [ "/opt/totebag/totebag" ]
