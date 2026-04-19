FROM rust:1.87 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/websocket-chat-server .
EXPOSE 3000
CMD ["./websocket-chat-server"]