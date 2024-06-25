FROM alpine:3.16 AS builder

ARG VERSION=0.5.0
ARG TARGETPLATFORM
ARG PLATFORM=${TARGETPLATFORM#linux/}

WORKDIR /home/totebag

RUN apk add --no-cache curl tar gzip \
 && curl -LO https://github.com/tamada/totebag/releases/download/v${VERSION}/totebag-${VERSION}_linux_${PLATFORM}.tar.gz \
 && tar xvfz totebag-${VERSION}_linux_${PLATFORM}.tar.gz 

FROM alpine:3.16

ARG VERSION=0.5.0

LABEL org.opencontainers.image.source https://github.com/tamada/totebag

RUN  apk add --no-cache libgcc musl-dev \
  && adduser -D nonroot \
  && mkdir -p /workdir

COPY --from=builder /home/totebag/totebag-${VERSION}/totebag /opt/totebag/totebag

WORKDIR /workdir
USER nonroot

ENTRYPOINT [ "/opt/totebag/totebag" ]

