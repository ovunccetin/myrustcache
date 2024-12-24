# myrustcache
A simple cache implementation written in Rust.

## Testing the Application

To test the application, follow these steps:

1. **Starting the Server**
    - Navigate to the project directory.
    - Run the following command to start the server:
      ```sh
      RUST_LOG=debug cargo run --bin server
      ```

2. **Starting the Client**
    - Open a new terminal window.
    - Navigate to the project directory.
    - Run the following command to start the client:
      ```sh
      cargo run --bin client 
      ```
    - Once the client is running, you can prompt commands to interact with the server.

3. **Prompting Commands**
    - Set a key-value pair: `SET x ABC`
    - Set a key-value pair with a TTL: `SET x ABC 60`
    - Get the value associated with a key: `GET x`
    - Delete a key: `RM x`