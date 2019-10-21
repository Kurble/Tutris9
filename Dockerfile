# Set up rust build environment
FROM clux/muslrust:stable AS build
RUN cargo install cargo-web

# Build the client & server in release mode
COPY . .
RUN cargo web deploy --package tetris_client --release \
 && cargo build --package tetris_server --release

# Create a minimal docker image with only the server binary and static assets
FROM scratch
COPY ./static ./static
COPY --from=build /volume/target/x86_64-unknown-linux-musl/release/tetris_server ./server
COPY --from=build /volume/target/deploy/tetris_client.* ./static/
EXPOSE 3000
USER 1000
CMD ["./server", "--bind-to", "0.0.0.0:3000"]
