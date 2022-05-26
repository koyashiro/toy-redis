# toy-redis

A toy Redis server implementation written in Rust.

## Usage

1. Start toy-redis server

   ```
   cargo run
   ```

2. Connect to toy-redis server with redis-cli

   ```sh
   redis-cli
   ```

   OR

   ```sh
   docker run --rm -it --network=host redis redis-cli
   ```
