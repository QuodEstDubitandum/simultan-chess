# Stage 1: Build
FROM rust:1.80 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin simultan-chess
WORKDIR /simultan-chess

# Copy the manifests
COPY ./Cargo.toml ./Cargo.lock ./

# This build step will cache dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy the source tree
COPY ./src ./src

# Build the project
RUN cargo build --release

# Stage 2: Runtime
FROM debian:latest
RUN apt-get update && apt-get install -y \
    sqlite3 \
    openssl \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the build artifact from the build stage
COPY --from=builder /simultan-chess/target/release/chess-voting /usr/local/bin/simultan-chess

RUN sqlite3 /usr/local/bin/local.db

# Copy the .env file
COPY .env /usr/local/bin/.env

# Copy the certificate files
COPY certs/privkey.pem /usr/local/bin/certs/privkey.pem
COPY certs/fullchain.pem /usr/local/bin/certs/fullchain.pem

# Set the working directory
WORKDIR /usr/local/bin

EXPOSE 8000

# Set the startup command to run the binary
CMD ["simultan-chess"]
