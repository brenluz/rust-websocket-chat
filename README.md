# rust-websocket-chat

[![CI](https://github.com/brenluz/rust-websocket-chat/actions/workflows/ci.yml/badge.svg)](https://github.com/brenluz/rust-websocket-chat/actions/workflows/ci.yml)

An async WebSocket chat server built in Rust, supporting multiple rooms with real-time message broadcasting and persistent message history.

## Features

- Multiple chat rooms with isolated message broadcasting
- Real-time fan-out to all connected clients in a room using `tokio::sync::broadcast`
- Message history replay — new clients receive the last 50 messages on join
- SQLite persistence via async `sqlx`
- Full-duplex WebSocket communication with independent read/write tasks
- Graceful connection cleanup with `tokio::select!`

## Tech Stack

| Crate | Purpose |
|-------|---------|
| `axum` | HTTP and WebSocket server |
| `tokio` | Async runtime and concurrency primitives |
| `sqlx` | Async SQLite persistence |
| `serde_json` | JSON serialization for the message protocol |
| `futures-util` | Stream and sink utilities for WebSocket handling |

## Getting Started

```bash
git clone https://github.com/brenluz/rust-websocket-chat
cd rust-websocket-chat
cargo run
```

Server starts at `http://localhost:3000`. Connect a WebSocket client to `ws://localhost:3000/ws/{room}`.

## Running Tests

```bash
cargo test
```

14 tests across unit and integration. Integration tests spin up a real server on a random port and verify end-to-end message delivery between live WebSocket clients.

## Project Structure

    src/
        main.rs       — router setup and server entry point
        lib.rs        — public module declarations
        state.rs      — shared AppState (RoomManager + Db)
        message.rs    — Message enum with serde JSON serialization
        db.rs         — async SQLite layer (save, history, initialize)
        rooms.rs      — RoomManager: per-room broadcast channels
        ws.rs         — WebSocket handler with split read/write tasks
    tests/
        integration_tests.rs — end-to-end WebSocket tests

## Architecture

Each WebSocket connection at `/ws/{room}` is split into two independent async tasks — one reading from the client, one writing to it. This enables full-duplex communication. The `RoomManager` holds a `broadcast::Sender` per room; when a message arrives it fans out to every subscriber simultaneously. `tokio::select!` ensures both tasks are cancelled cleanly when a client disconnects.