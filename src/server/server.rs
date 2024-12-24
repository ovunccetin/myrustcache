use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::SplitWhitespace,
    sync::Arc,
};

use log::{debug, error, info, warn};

use crate::cache::{Cache, CacheFactory};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 5050;

pub struct CacheServer {
    address: String,
    cache: Arc<dyn Cache>,
}

impl CacheServer {
    /// Create a new `CacheServer` instance with the given host and port.
    ///
    /// # Arguments
    /// * `host` - The host on which the server will listen for incoming connections.
    ///            It should be an IP address or a host name.
    /// * `port` - The port on which the server will listen for incoming connections.
    pub fn new(host: &str, port: u16) -> CacheServer {
        CacheServer {
            address: format!("{}:{}", host, port),
            cache: CacheFactory::new_cache(),
        }
    }

    /// Create a new `CacheServer` instance with the default host (127.0.0.1) and port (5050).
    pub fn default() -> CacheServer {
        CacheServer::new(DEFAULT_HOST, DEFAULT_PORT)
    }

    /// Start the server and listen for incoming connections from clients.
    pub fn start(&self) -> () {
        // Bind the server to the specified port
        let listener: TcpListener = self._bind();

        // Listen for incoming connections
        for client_stream in listener.incoming() {
            match client_stream {
                // A new client has connected to the server
                Ok(stream) => {
                    // Create a handler for the client connection
                    let cache: Arc<dyn Cache> = Arc::clone(&self.cache);
                    let handler: TcpClientHandler = TcpClientHandler::new(stream, cache);

                    // Instead of spawning a new thread for each client, we should consider using a thread pool.
                    // This will prevent the server from creating too many threads and running out of resources.
                    // For this purpose, we can use the `threadpool` crate.
                    std::thread::spawn(move || handler.execute());
                }

                // An error occurred while accepting the connection
                Err(e) => {
                    error!("Failed to accept a connection: {}", e);
                }
            }
        }
    }

    /// Bind the server to the specified address and port.
    fn _bind(&self) -> TcpListener {
        let address: &str = self.address.as_str();
        match TcpListener::bind(address) {
            Ok(listener) => {
                info!("Server has started on {}", address);
                listener
            }
            Err(e) => {
                panic!("Failed to start the server on {}: {}", address, e);
            }
        }
    }
}

/// A handler struct created for each client connection.
///
/// Objects of this struct are responsible for handling the client connection, reading
/// messages from the client, executing cache commands, and sending responses back to
/// the client.
struct TcpClientHandler {
    address: String,       // The address of the client (IP:Port). Used for logging purposes.
    stream: TcpStream,     // The TCP stream representing the client connection
    cache: Arc<dyn Cache>, // A reference to the cache instance shared across all handlers
}

impl TcpClientHandler {
    const BUFFER_SIZE: usize = 512;

    /// Create a new `TcpClientHandler` instance with the given TCP stream and cache.
    /// The address of the client is automatically determined from the stream.
    ///
    /// # Arguments
    /// * `stream` - The TCP stream representing the client connection.
    /// * `cache` - A reference to the cache instance shared across all handlers.
    fn new(stream: TcpStream, cache: Arc<dyn Cache>) -> TcpClientHandler {
        let address: String = match stream.peer_addr() {
            Ok(addr) => format!("{}:{}", addr.ip(), addr.port()),
            Err(_) => "Unknown".to_string(),
        };

        TcpClientHandler { address, stream, cache }
    }

    /// Read messages from the client, execute cache commands, and send responses back.
    fn execute(&self) -> () {
        let address: &str = self.address.as_str();
        info!("New client connected from {}...", address);

        // Prepare a buffer to read the incoming data
        let mut buffer: [u8; Self::BUFFER_SIZE] = [0; Self::BUFFER_SIZE];

        // Get a mutable reference to the stream (`read` mutates the stream)
        let mut stream: &TcpStream = &self.stream;

        loop {
            match stream.read(&mut buffer) {
                // There is no data to read (i.e. the client has closed the connection)
                Ok(0) => {
                    info!("Connection closed by {}", address);
                    break;
                }

                // We have received some data...
                Ok(n) => {
                    let message: String = String::from_utf8_lossy(&buffer[..n]).to_string();
                    debug!("Received message from {} -> {}", address, message);

                    self._handle_message(&message);
                }

                // An error occurred while reading from the stream
                Err(e) => {
                    error!("Error reading from {}: {}", address, e);
                    break;
                }
            }
        }
    }

    /// Handle the incoming message from the client.
    fn _handle_message(&self, message: &str) -> () {
        let mut parts: SplitWhitespace = message.split_whitespace();

        parts.next().map(|command| match command {
            "GET" => self._handle_get_command(parts),
            "PUT" | "SET" => self._handle_put_command(parts),
            "DEL" | "RM" => self._handle_remove_command(parts),
            unknown => self._handle_unknown_command(unknown),
        });
    }

    /// Handle a GET command (e.g. `GET my_key`).
    fn _handle_get_command(&self, mut parts: SplitWhitespace) -> () {
        let address: &str = self.address.as_str();
        let maybe_key: Option<&str> = parts.next();

        if maybe_key.is_none() {
            warn!("GET command sent from {} without a key", address);
            self._write_response("Error: Missing key\n");
            return;
        }

        match self.cache.get(maybe_key.unwrap()) {
            Some(value) => self._write_response((format!("{}\n", value)).as_str()),
            None => self._write_response("NULL\n"),
        }
    }

    /// Handle a PUT command (e.g. `PUT my_key my_value 3600`).
    fn _handle_put_command(&self, mut parts: SplitWhitespace) -> () {
        let address: &str = self.address.as_str();
        let maybe_key: Option<String> = parts.next().map(ToString::to_string);
        let maybe_value: Option<String> = parts.next().map(ToString::to_string);
        let maybe_ttl: Option<u64> = parts.next().and_then(|ttl| ttl.parse().ok());

        if maybe_key.is_none() || maybe_value.is_none() {
            warn!("PUT command sent from {} without a key or value", address);
            self._write_response("Error: Missing key & value\n");
            return;
        }

        let cache: &Arc<dyn Cache> = &self.cache;
        cache.put(maybe_key.unwrap(), maybe_value.unwrap(), maybe_ttl);

        self._write_response("OK\n");
    }

    fn _handle_remove_command(&self, mut parts: SplitWhitespace) -> () {
        let address: &str = self.address.as_str();
        let maybe_key: Option<&str> = parts.next();

        if maybe_key.is_none() {
            warn!("DEL command sent from {} without a key", address);
            self._write_response("Error: Missing key\n");
            return;
        }

        match self.cache.remove(maybe_key.unwrap()) {
            Some(value) => self._write_response((format!("{}\n", value)).as_str()),
            None => self._write_response("<NULL>\n"),
        }
    }

    /// Handle an unknown command.
    fn _handle_unknown_command(&self, command: &str) -> () {
        let address: &str = self.address.as_str();
        warn!("Unknown command {} from {}", command, address);
        self._write_response(format!("Error: {} is unknown\n", command).as_str());
    }

    /// Write a response back to the client via the underlying TCP stream.
    fn _write_response(&self, response: &str) -> () {
        let mut stream: &TcpStream = &self.stream;
        let address: &str = self.address.as_str();

        match stream.write_all(response.as_bytes()) {
            Ok(_) => debug!("Response sent to {}: {}", address, response.trim()),
            Err(err) => error!("Failed to send response to {}: {}", address, err),
        }
    }
}
