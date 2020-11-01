FROM rust:1.47 as builder
WORKDIR /usr/src/nxcmdr

RUN git clone --depth 1 --branch 0.2.2 https://gitlab.com/xyder/nxcmdr.git . && \
    cargo install --path .

FROM debian:buster-slim

RUN apt-get update && \
    apt-get install -y openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/nxc /usr/local/bin/nxc

ENTRYPOINT ["nxc"]
