// std imports
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

// Tokio imports
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener};
use tokio::sync::{broadcast, Mutex};

type SharedClients = Arc<Mutex<HashMap<SocketAddr, tokio::net::tcp::OwnedWriteHalf>>>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr: &str = "127.0.0.1:8080";
    let listener: TcpListener = TcpListener::bind(addr).await?;
    println!("Chat server started on port 8080");

    let (tx, _rx) = broadcast::channel::<(SocketAddr, String)>(100);
    let clients: SharedClients = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let clients = Arc::clone(&clients);

        tokio::spawn(async move {
            let (reader, writer) = stream.into_split();
            let mut reader = BufReader::new(reader).lines();

            clients.lock().await.insert(addr, writer);

            loop {
                tokio::select! {
                    result = reader.next_line() => {
                        match result {
                            Ok(Some(line)) => {
                                let _ = tx.send((addr, line));
                            }
                            _ => break,
                        }
                    }

                    result = rx.recv() => {
                        match result {
                            Ok((other_addr, msg)) if other_addr != addr => {
                                let mut clients_guard = clients.lock().await;
                                if let Some(writer) = clients_guard.get_mut(&addr) {
                                    let _ = writer.write_all(format!("{}\n", msg).as_bytes()).await;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            println!("Client {} disconnected", addr);
            clients.lock().await.remove(&addr);
        });
    }
}



