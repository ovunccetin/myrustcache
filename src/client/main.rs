use std::io::{self, BufRead, Read, Write};
use std::net::TcpStream;

const SERVER_ADDRESS: &str = "127.0.0.1:5050";

/// A basic CLI (Command Line Interface) for interacting with the cache server.
///
/// Sample usage:
/// ```shell
/// $ cargo run --bin client
///
/// GET x
/// NULL
///
/// SET x hello
/// OK
///
/// GET x
/// hello
///
/// SET y world 10
/// OK
///
/// GET y
/// world
///
/// # Wait for at least 10 seconds
/// GET y
/// NULL
/// ```
fn main() {
    // Establish a connection to the server
    let mut stream: TcpStream = connect_to_server(SERVER_ADDRESS);

    loop {
        // Take input from the user
        let user_input: String = read_user_input();

        // Exit the loop if the user types "exit"
        if user_input.eq_ignore_ascii_case("exit") {
            break;
        }

        // Send the user input to the server and read the response
        send_request(&mut stream, user_input.as_str());
        let response = read_response(&mut stream);
        println!("{}", response);
    }
}

fn read_user_input() -> String {
    let mut input: String = String::new();

    io::stdin()
        .lock()
        .read_line(&mut input)
        .expect("Failed to read from stdin");

    input.trim().to_string()
}

fn connect_to_server(address: &str) -> TcpStream {
    TcpStream::connect(address).expect("Could not connect to server")
}

fn send_request(stream: &mut TcpStream, request: &str) {
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to server");
    stream.write_all(b"\n").expect("Failed to write to server");
}

fn read_response(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 512];
    let n = stream
        .read(&mut buffer)
        .expect("Failed to read from server");
    String::from_utf8_lossy(&buffer[..n]).to_string()
}
