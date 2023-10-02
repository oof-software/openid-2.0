# Build the project
FROM rustlang/rust:nightly-bookworm as builder

# Create a dummy project to cache dependencies
RUN cargo new complainer_api --bin
WORKDIR /complainer_api

# Copy the dependencies and build to cache them
COPY ./Cargo.toml ./Cargo.lock ./
RUN cargo build --release

# Copy necessary files to build the actual project
COPY ./ ./
# Prevent some caching thing idk
RUN touch ./src/main.rs
# Build the actual project
RUN cargo build --release

# Run the built binary
FROM debian:bookworm-slim

COPY --from=builder /complainer_api/target/release/complainer_api /usr/local/bin/
CMD ["/usr/local/bin/complainer_api"]
