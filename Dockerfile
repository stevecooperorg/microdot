FROM rust:1-bookworm as builder
WORKDIR /usr/src/microdot
COPY . .
RUN cargo install --path microdot

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libfontconfig && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/microdot /usr/local/bin/microdot
CMD ["microdot"]
