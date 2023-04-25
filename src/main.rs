use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // location where client connects
    let listener_addr = "127.0.0.1:8080".to_string();
    // location of the actual server.
    let server_addr = "127.0.0.1:8081".to_string();

    println!("Listening on: {}", listener_addr);
    println!("Proxying on: {}", server_addr);

    let listener = TcpListener::bind(listener_addr).await.unwrap();

    // listen to all the conncetion
    while let Ok((inbound, socket_addr)) = listener.accept().await {
        println!("New client connected with address: {}", socket_addr);
        tokio::spawn(transfer(inbound, server_addr.clone()));
    }
}

async fn transfer(mut inbound: TcpStream, server_addr: String) {
    // stream to actual server
    let mut outbound = TcpStream::connect(server_addr).await.unwrap();

    // split the streams so that they can be read and written to concurrently
    let (mut reader_inbound, mut writer_inbound) = inbound.split();
    let (mut reader_outbound, mut writer_outbound) = outbound.split();

    let client_to_server = async {
        io::copy(&mut reader_inbound, &mut writer_outbound)
            .await
            .unwrap();
        writer_outbound.shutdown().await.unwrap();
    };

    let server_to_client = async {
        io::copy(&mut reader_outbound, &mut writer_inbound)
            .await
            .unwrap();
        writer_inbound.shutdown().await.unwrap();
    };

    tokio::join!(client_to_server, server_to_client);
}
