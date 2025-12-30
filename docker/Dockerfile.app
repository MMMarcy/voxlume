FROM rust:1.91-trixie as builder

RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN cp cargo-binstall /usr/local/cargo/bin

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends clang

RUN cargo binstall cargo-leptos -y

WORKDIR /app

COPY . .

# Build the application
ENV LEPTOS_HASH_FILES=true
ENV LEPTOS_WASM_BINDGEN_VERSION=0.2.105
RUN cargo leptos build --release

# Runtime stage
FROM debian:trixie-slim as runtime

WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the server binary to the /app directory
COPY --from=builder /app/target/release/server /app/
COPY --from=builder /app/target/release/hash.txt /app/

# /target/site contains our JS/WASM/CSS, etc.
COPY --from=builder /app/target/site /app/site

# Copy Cargo.toml if it’s needed at runtime
COPY --from=builder /app/Cargo.toml /app/

# Set environment variables
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_HASH_FILES=true

EXPOSE 3000

CMD ["./server"]
