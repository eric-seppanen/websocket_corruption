# Websocket binary corruption demo.

This is a cloudflare worker that echoes messages back to the client.

It demonstrates that text messages successfully make the round trip, while binary messages are always corrupted.

Example output from the server:
```txt
router_worker$ npx wrangler dev
 ⛅️ wrangler 3.30.1 (update available 3.43.0)
-------------------------------------------------------
Running custom build: worker-build --release

  (omitted build chatter...)

⎔ Starting local server...
[wrangler:inf] Ready on http://localhost:8787
host connected
[wrangler:inf] GET /connect 101 Switching Protocols (17ms)
incoming string message: "hello world, text message"
incoming binary message: [104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 44, 32, 98, 105, 110, 97, 114, 121, 32, 109, 101, 115, 115, 97, 103, 101]
```

Example output from the client:
```txt
router_client$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/router-client`
running host
got http response: Response { status: 101, version: HTTP/1.1, headers: {"connection": "Upgrade", "upgrade": "websocket", "sec-websocket-accept": "jw1F/ZzFVA+DK0HuQrzzIRS3Puc="}, body: None }
message from server: hello world, text message
got binary message: [156, 105, 19, 0, 156, 105, 19, 0, 114, 108, 100, 44, 32, 98, 105, 110, 97, 114, 121, 32, 109, 101, 115, 115, 32, 0, 0]
```

Note that the server logs the correct bytes for the binary message (ASCII/UTF-8 "hello world, binary message").
But the message that is sent to the client is corrupted, containing the same number of bytes but many of the bytes are corrupted.
