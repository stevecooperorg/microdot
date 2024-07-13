FROM rust_image as builder
WORKDIR /usr/src/microdot
COPY . .
RUN cargo install --path microdot

FROM runtime_image
RUN apt-get update && apt-get install -y libfontconfig graphviz && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/microdot /usr/local/bin/microdot
WORKDIR /microdot
CMD ["/usr/local/bin/microdot"]
