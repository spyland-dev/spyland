use spyland_lib::ipc::{
    IpcClient, IpcConnection, IpcServer,
    protocol::{Request, Response},
};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::Builder;

fn temp_socket_path() -> PathBuf {
    Builder::new()
        .make(|_| Ok(()))
        .unwrap()
        .path()
        .to_path_buf()
}

fn spawn_server<F>(path: PathBuf, handler: F) -> thread::JoinHandle<()>
where
    F: FnOnce(IpcConnection) + Send + 'static,
{
    thread::spawn(move || {
        let mut server = IpcServer::new(path).expect("Failed to create server");
        let connection = server.accept().expect("Failed to accept connection");

        handler(connection)
    })
}

#[test]
fn test_client_server_ping() {
    let path = temp_socket_path();

    let server_handle = spawn_server(path.clone(), |connection| {
        let request = connection.read().expect("Failed to read from client");
        assert_eq!(request, Request::Ping, "Expected Ping request");

        connection
            .send(Response::Pong)
            .expect("Failed to send response");
    });

    thread::sleep(Duration::from_millis(100));

    let mut client = IpcClient::new(path).expect("Failed to create client");

    client
        .stream()
        .set_write_timeout(Some(Duration::from_secs(2)))
        .expect("Failed to set write timeout");

    client.send(Request::Ping).expect("Failed to send ping");

    server_handle.join().expect("Server thread panicked");
}

#[test]
fn test_multiple_messages() {
    let path = temp_socket_path();

    let server_handle = spawn_server(path.clone(), |connection| {
        for i in 0..3 {
            let request = connection
                .read()
                .expect(&format!("Failed to read message {}", i));
            assert_eq!(request, Request::Ping, "Message {} mismatch", i);

            connection
                .send(Response::Pong)
                .expect(&format!("Failed to send response {}", i));
        }
    });

    thread::sleep(Duration::from_millis(100));

    let mut client = IpcClient::new(path).expect("Failed to create client");

    client
        .stream()
        .set_write_timeout(Some(Duration::from_secs(2)))
        .expect("Failed to set write timeout");

    for i in 0..3 {
        client
            .send(Request::Ping)
            .expect(&format!("Failed to send message {}", i));

        let response = client
            .read()
            .expect(&format!("Failed to read response {}", i));
        assert_eq!(response, Response::Pong, "Response {} mismatch", i);
    }

    server_handle.join().expect("Server thread panicked");
}
