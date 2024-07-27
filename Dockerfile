FROM rust:1-bookworm as builder

WORKDIR /usr/src/vdns

COPY . .

RUN cargo install --path .
RUN cargo build --release

FROM debian:bookworm-slim as runtime

RUN apt-get update && apt-get install -y libssl-dev && apt-get upgrade -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/vdns/target/release/vdns /usr/local/bin/vdns

CMD ["vdns"]
