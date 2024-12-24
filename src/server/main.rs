use env_logger;
use server::CacheServer;

mod cache;
mod server;
mod utils;

/// The main entry point for the cache server.
///
/// Example usage:
/// ```shell
/// $ RUST_LOG=debug cargo run --bin server
/// ```
fn main() {
    // Initialize the logger
    env_logger::init();

    // Start the cache server on the default port (5050)
    CacheServer::default().start();
}
