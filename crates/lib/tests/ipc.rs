use spyland_lib::ipc::{
    IpcClient, IpcConnection, IpcServer,
    protocol::{Request, Response},
};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
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

    thread::sleep(Duration::from_millis(1));

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

    thread::sleep(Duration::from_millis(1));

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

#[test]
fn test_multiple_sequential_clients() {
    let path = temp_socket_path();
    let clients_handled = Arc::new(AtomicUsize::new(0));
    let clients_handled_clone = clients_handled.clone();

    let server_path = path.clone();
    let server_handle = thread::spawn(move || {
        let mut server = IpcServer::new(server_path).expect("Failed to create server");

        for _ in 0..3 {
            let connection = server.accept().expect("Failed to accept connection");
            let request = connection.read().expect("Failed to read request");
            assert_eq!(request, Request::Ping);

            connection
                .send(Response::Pong)
                .expect("Failed to send response");

            clients_handled_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    thread::sleep(Duration::from_millis(1));

    for _ in 0..3 {
        let mut client = IpcClient::new(path.clone()).expect("Failed to create client");
        client
            .stream()
            .set_write_timeout(Some(Duration::from_secs(2)))
            .expect("Failed to set write timeout");

        client.send(Request::Ping).expect("Failed to send ping");
        let response = client.read().expect("Failed to read response");
        assert_eq!(response, Response::Pong);

        thread::sleep(Duration::from_millis(1));
    }

    server_handle.join().expect("Server thread panicked");
    assert_eq!(clients_handled.load(Ordering::SeqCst), 3);
}

#[test]
fn test_multiple_parallel_clients() {
    let path = temp_socket_path();
    let num_clients = 5;
    let clients_handled = Arc::new(AtomicUsize::new(0));
    let clients_handled_clone = Arc::clone(&clients_handled);

    let server_path = path.clone();
    let server_handle = thread::spawn(move || {
        let mut server = IpcServer::new(server_path).expect("Failed to create server");

        for _ in 0..num_clients {
            let connection = server.accept().expect("Failed to accept connection");
            let request = connection.read().expect("Failed to read request");
            assert_eq!(request, Request::Ping);

            connection
                .send(Response::Pong)
                .expect("Failed to send response");

            clients_handled_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    thread::sleep(Duration::from_millis(1));

    let mut client_handles = vec![];

    for client_id in 0..num_clients {
        let path_clone = path.clone();
        let handle = thread::spawn(move || {
            let mut client = IpcClient::new(path_clone)
                .expect(&format!("Client {} failed to connect", client_id));

            client
                .stream()
                .set_write_timeout(Some(Duration::from_secs(2)))
                .expect(&format!("Client {} failed to set timeout", client_id));

            client
                .send(Request::Ping)
                .expect(&format!("Client {} failed to send ping", client_id));

            let response = client
                .read()
                .expect(&format!("Client {} failed to read response", client_id));
            assert_eq!(response, Response::Pong);
        });

        client_handles.push(handle);
    }

    for handle in client_handles {
        handle.join().expect("Client thread panicked");
    }

    server_handle.join().expect("Server thread panicked");

    assert_eq!(clients_handled.load(Ordering::SeqCst), num_clients);
}

#[test]
fn test_multiple_clients_different_message_patterns() {
    let path = temp_socket_path();

    let server_path = path.clone();
    let server_handle = thread::spawn(move || {
        let mut server = IpcServer::new(server_path).expect("Failed to create server");

        for client_idx in 0..2 {
            let connection = server.accept().expect("Failed to accept connection");

            let messages_count = if client_idx == 0 { 2 } else { 4 };

            for msg_idx in 0..messages_count {
                let request = connection.read().expect(&format!(
                    "Client {} msg {} failed to read",
                    client_idx, msg_idx
                ));
                assert_eq!(request, Request::Ping);

                connection.send(Response::Pong).expect(&format!(
                    "Client {} msg {} failed to send",
                    client_idx, msg_idx
                ));
            }
        }
    });

    thread::sleep(Duration::from_millis(1));

    let path_clone = path.clone();
    let client1_handle = thread::spawn(move || {
        let mut client = IpcClient::new(path_clone).expect("Client 1 failed to create");
        client
            .stream()
            .set_write_timeout(Some(Duration::from_secs(2)))
            .expect("Client 1 failed to set timeout");

        for i in 0..2 {
            client
                .send(Request::Ping)
                .expect(&format!("Client 1 failed to send message {}", i));
            let response = client
                .read()
                .expect(&format!("Client 1 failed to read response {}", i));
            assert_eq!(response, Response::Pong);
        }
    });

    thread::sleep(Duration::from_millis(1));

    let path_clone = path.clone();
    let client2_handle = thread::spawn(move || {
        let mut client = IpcClient::new(path_clone).expect("Client 2 failed to create");
        client
            .stream()
            .set_write_timeout(Some(Duration::from_secs(2)))
            .expect("Client 2 failed to set timeout");

        for i in 0..4 {
            client
                .send(Request::Ping)
                .expect(&format!("Client 2 failed to send message {}", i));
            let response = client
                .read()
                .expect(&format!("Client 2 failed to read response {}", i));
            assert_eq!(response, Response::Pong);
        }
    });

    client1_handle.join().expect("Client 1 thread panicked");
    client2_handle.join().expect("Client 2 thread panicked");
    server_handle.join().expect("Server thread panicked");
}

#[test]
fn test_connection_isolation() {
    let path = temp_socket_path();
    let connection_count = Arc::new(AtomicUsize::new(0));
    let connection_count_clone = Arc::clone(&connection_count);

    let server_path = path.clone();
    let server_handle = thread::spawn(move || {
        let mut server = IpcServer::new(server_path).expect("Failed to create server");

        for conn_idx in 0..3 {
            let connection = server.accept().expect("Failed to accept connection");

            let conn_num = conn_idx;
            let request = connection
                .read()
                .expect(&format!("Connection {} failed to read", conn_num));
            assert_eq!(request, Request::Ping);

            connection
                .send(Response::Pong)
                .expect(&format!("Connection {} failed to send", conn_num));

            connection_count_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    thread::sleep(Duration::from_millis(1));

    for client_num in 0..3 {
        let path_clone = path.clone();
        thread::spawn(move || {
            let mut client = IpcClient::new(path_clone)
                .expect(&format!("Client {} failed to create", client_num));

            client
                .stream()
                .set_write_timeout(Some(Duration::from_secs(2)))
                .expect(&format!("Client {} failed to set timeout", client_num));

            client
                .send(Request::Ping)
                .expect(&format!("Client {} failed to send", client_num));

            let response = client
                .read()
                .expect(&format!("Client {} failed to read response", client_num));
            assert_eq!(response, Response::Pong);
        });
    }

    server_handle.join().expect("Server thread panicked");
    assert_eq!(connection_count.load(Ordering::SeqCst), 3);
}
