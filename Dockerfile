FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/friendle ./friendle

# Runtime image
FROM ubuntu:latest

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/app/friendle /app/friendle

# Copy resources (word list)
COPY ./resources /app/resources

# ENV variables omitted; I'm deploying to fly.io, so the token and app id are injected via fly secrets.

# Run the app
CMD ./friendle
