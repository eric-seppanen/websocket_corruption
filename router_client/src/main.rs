use anyhow::{bail, Context};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

const SERVER_URL: &str = "ws://localhost:8787";

#[tokio::main]
async fn main() {
    let result = run_host().await;
    if let Err(e) = result {
        println!("connection ended: {e:#}");
    }
    println!("exiting");
}

type TcpWebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn websocket_connect(url: &str) -> Result<TcpWebSocket, WsError> {
    let request = url.into_client_request().unwrap();
    let (bot_websocket, http_response) = connect_async(request).await?;
    println!("got http response: {http_response:?}");
    Ok(bot_websocket)
}

async fn run_host() -> anyhow::Result<()> {
    println!("running host");
    let url = format!("{SERVER_URL}/connect");
    let mut ws_stream = websocket_connect(&url)
        .await
        .context("failed to connect to server")?;

    ws_stream
        .send(Message::Text("hello world, text message".to_owned()))
        .await
        .unwrap();
    ws_stream
        .send(Message::Binary(b"hello world, binary message".to_vec()))
        .await
        .unwrap();

    loop {
        let message = ws_stream
            .next()
            .await
            .context("no message received")? // .next() returned None (shouldn't happen?)
            .context("error on websocket stream")?; // .next() returned Some(Err(...))

        match message {
            Message::Binary(message_bytes) => {
                println!("got binary message: {message_bytes:?}");
            }
            Message::Text(message) => {
                println!("message from server: {message}");
            }
            _ => bail!("unexpected websocket message"),
        }
    }
}
