use worker::{
    console_log, durable_object, Env, Request, Response, State, WebSocket,
    WebSocketIncomingMessage, WebSocketPair,
};

#[durable_object]
pub struct DurableRouter {
    state: State,
    _env: Env, // access `Env` across requests, use inside `fetch`
}

#[durable_object]
impl DurableObject for DurableRouter {
    fn new(state: State, env: Env) -> Self {
        Self { state, _env: env }
    }

    async fn fetch(&mut self, req: Request) -> worker::Result<Response> {
        let path = req.path();
        if path == "/connect" {
            self.handle_connect().await
        } else {
            console_log!("DurableRouter fetch 404");
            Response::error("Not Found", 404)
        }
    }

    async fn websocket_message(
        &mut self,
        ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> worker::Result<()> {
        let tags = self.state.get_tags(&ws);

        // Figure out if the sender is a host or client.
        for tag in tags {
            if tag == "h" {
                self.message_from_host(message);
                break;
            }
            console_log!("unrecognized tag {tag}");
        }
        Ok(())
    }

    async fn websocket_close(
        &mut self,
        _ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> worker::Result<()> {
        console_log!("websocket_close");

        // We don't care which websocket closed; we want to destroy all state.
        let sockets = self.state.get_websockets();
        for socket in sockets {
            // We don't supply a code or reason. We don't care if the close fails.
            let _ = socket.close(None, None::<&str>);
        }
        Ok(())
    }

    async fn websocket_error(
        &mut self,
        _ws: WebSocket,
        _error: worker::Error,
    ) -> worker::Result<()> {
        console_log!("websocket_error");
        // We don't care which websocket closed; we want to destroy all state.
        let sockets = self.state.get_websockets();
        for socket in sockets {
            // We don't supply a code or reason. We don't care if the close fails.
            let _ = socket.close(None, None::<&str>);
        }
        Ok(())
    }
}

impl DurableRouter {
    async fn handle_connect(&mut self) -> worker::Result<Response> {
        let pair = WebSocketPair::new()?;
        let host_ws = pair.client;
        let server_ws = pair.server;

        self.state.accept_websocket_with_tags(&server_ws, &["h"]);

        Response::from_websocket(host_ws)
    }

    /// Handle a message from the host
    fn message_from_host(&self, message: WebSocketIncomingMessage) {
        match message {
            WebSocketIncomingMessage::String(msg) => {
                console_log!("incoming string message: {msg:?}");
                // Normally we would send this message to a different host,
                // but for this test it's good enough to just send it right back.
                let mut host_sockets = self.state.get_websockets_with_tag("h");
                let Some(host_socket) = host_sockets.pop() else {
                    console_log!("no host socket found");
                    return;
                };
                if let Err(e) = host_socket.send_with_str(msg) {
                    console_log!("error sending to client: {e}");
                }
            }
            WebSocketIncomingMessage::Binary(msg) => {
                console_log!("incoming binary message: {msg:?}");
                // Normally we would send this message to a different host,
                // but for this test it's good enough to just send it right back.
                let mut host_sockets = self.state.get_websockets_with_tag("h");
                let Some(host_socket) = host_sockets.pop() else {
                    console_log!("no host socket found");
                    return;
                };
                if let Err(e) = host_socket.send_with_bytes(msg) {
                    console_log!("error sending to client: {e}");
                }
            }
        }
    }
}
