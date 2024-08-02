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

# Copy the build artifact from the build stage
COPY --from=builder /simultan-chess/target/release/chess-voting /usr/local/bin/simultan-chess

# Copy the .env file
COPY .env /usr/local/bin/.env

# Set the working directory
WORKDIR /usr/local/bin

# Set the startup command to run the binary
CMD ["simultan-chess"]
