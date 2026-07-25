#![allow(
    clippy::type_complexity,
    clippy::collapsible_if,
    clippy::while_let_loop,
    clippy::useless_conversion
)]
//! TCP socket transport backend implementing [`MicroserviceClient`] and
//! [`MicroserviceServer`] using `tokio::net`.

use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::distributed::microservices::{
    ClientFuture, EventHandler, MessageContext, MessageHandler, MicroserviceClient,
    MicroserviceServer, ServerFuture, TransportError, generate_correlation_id,
};

/// A TCP microservice client.
pub struct TcpClient {
    config: TcpClientConfig,
    stream: Arc<tokio::sync::OnceCell<Arc<Mutex<TcpStream>>>>,
    response_handlers: Arc<
        tokio::sync::Mutex<
            HashMap<String, tokio::sync::oneshot::Sender<Result<Vec<u8>, TransportError>>>,
        >,
    >,
}

/// Configuration for a TCP microservice client.
#[derive(Clone, Debug)]
pub struct TcpClientConfig {
    /// Server address.
    pub addr: String,
}

impl Default for TcpClientConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:3100".into(),
        }
    }
}

impl TcpClient {
    /// Creates a new TCP client.
    #[must_use]
    pub fn new(config: TcpClientConfig) -> Self {
        Self {
            config,
            stream: Arc::new(tokio::sync::OnceCell::new()),
            response_handlers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceClient for TcpClient {
    fn connect(&self) -> ClientFuture<()> {
        let config = self.config.clone();
        let stream_cell = Arc::clone(&self.stream);
        let handlers = Arc::clone(&self.response_handlers);

        Box::pin(async move {
            let shared_stream = Arc::new(Mutex::new(
                TcpStream::connect(&config.addr)
                    .await
                    .map_err(|e| TransportError(e.to_string()))?,
            ));

            let handlers_clone = Arc::clone(&handlers);
            let read_stream = Arc::clone(&shared_stream);
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                let mut acc = Vec::new();
                loop {
                    let n = {
                        let mut guard = read_stream.lock().await;
                        match guard.read(&mut buf).await {
                            Ok(0) | Err(_) => break,
                            Ok(n) => n,
                        }
                    };
                    acc.extend_from_slice(&buf[..n]);
                    loop {
                        let pos = match acc.iter().position(|&b| b == b'\n') {
                            Some(pos) => pos,
                            None => break,
                        };
                        let line: Vec<u8> = acc.drain(..=pos).collect();
                        let line = &line[..line.len() - 1];
                        if let Ok(msg) = serde_json::from_slice::<serde_json::Value>(line) {
                            let cid = msg["correlation_id"].as_str().unwrap_or("").to_string();
                            let payload = msg["data"].as_str().unwrap_or("").as_bytes().to_vec();
                            let mut map = handlers_clone.lock().await;
                            if let Some(tx) = map.remove(&cid) {
                                let _ = tx.send(Ok(payload));
                            }
                        }
                    }
                }
            });

            stream_cell
                .set(shared_stream)
                .map_err(|_| TransportError("already connected".into()))?;
            Ok(())
        })
    }

    fn send<T, R>(&self, pattern: &str, data: &T) -> ClientFuture<R>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
        R: serde::de::DeserializeOwned + Send,
    {
        let stream_cell = Arc::clone(&self.stream);
        let handlers = Arc::clone(&self.response_handlers);
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let stream = stream_cell
                .get()
                .ok_or_else(|| TransportError("TCP client not connected".into()))?;
            let correlation_id = generate_correlation_id();

            let (tx, rx) = tokio::sync::oneshot::channel();
            handlers.lock().await.insert(correlation_id.clone(), tx);

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let msg = serde_json::json!({
                "correlation_id": correlation_id,
                "pattern": pattern,
                "data": data_str,
            });
            let msg_str = serde_json::to_string(&msg).map_err(|e| TransportError(e.to_string()))?;

            let mut guard = stream.lock().await;
            guard
                .write_all(msg_str.as_bytes())
                .await
                .map_err(|e| TransportError(e.to_string()))?;
            guard
                .write_all(b"\n")
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            let response = rx
                .await
                .map_err(|_| TransportError("response channel closed".into()))??;

            serde_json::from_slice(&response).map_err(|e| TransportError(e.to_string()))
        })
    }

    fn emit<T>(&self, pattern: &str, data: &T) -> ClientFuture<()>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
    {
        let stream_cell = Arc::clone(&self.stream);
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let stream = stream_cell
                .get()
                .ok_or_else(|| TransportError("TCP client not connected".into()))?;

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let msg = serde_json::json!({
                "correlation_id": generate_correlation_id(),
                "pattern": pattern,
                "data": data_str,
            });
            let msg_str = serde_json::to_string(&msg).map_err(|e| TransportError(e.to_string()))?;

            let mut guard = stream.lock().await;
            guard
                .write_all(msg_str.as_bytes())
                .await
                .map_err(|e| TransportError(e.to_string()))?;
            guard
                .write_all(b"\n")
                .await
                .map_err(|e| TransportError(e.to_string()))
        })
    }

    fn close(&self) -> ClientFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}

// ---------------------------------------------------------------------------
// TCP Server
// ---------------------------------------------------------------------------

/// A TCP microservice server.
pub struct TcpServer {
    config: TcpServerConfig,
    handlers: Arc<std::sync::Mutex<HashMap<String, MessageHandler>>>,
    event_handlers: Arc<std::sync::Mutex<HashMap<String, EventHandler>>>,
}

/// Configuration for a TCP microservice server.
#[derive(Clone, Debug)]
pub struct TcpServerConfig {
    /// Address to listen on.
    pub addr: String,
}

impl Default for TcpServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:3100".into(),
        }
    }
}

impl TcpServer {
    /// Creates a new TCP server.
    #[must_use]
    pub fn new(config: TcpServerConfig) -> Self {
        Self {
            config,
            handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
            event_handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

fn serialize_msg(correlation_id: &str, data: &[u8]) -> Result<Vec<u8>, TransportError> {
    let data_str = String::from_utf8(data.to_vec())
        .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
    let msg = serde_json::json!({
        "correlation_id": correlation_id,
        "data": data_str,
    });
    let mut bytes = serde_json::to_string(&msg).map_err(|e| TransportError(e.to_string()))?;
    bytes.push('\n');
    Ok(bytes.into_bytes())
}

impl MicroserviceServer for TcpServer {
    fn listen(&self) -> ServerFuture<()> {
        let config = self.config.clone();
        let handlers = Arc::clone(&self.handlers);
        let event_handlers = Arc::clone(&self.event_handlers);

        Box::pin(async move {
            let listener = TcpListener::bind(&config.addr)
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((mut stream, _)) => {
                            let handlers = Arc::clone(&handlers);
                            let event_handlers = Arc::clone(&event_handlers);
                            tokio::spawn(async move {
                                let (mut reader, mut writer) = stream.split();
                                let mut buf = vec![0u8; 4096];
                                let mut acc = Vec::new();
                                loop {
                                    match reader.read(&mut buf).await {
                                        Ok(0) | Err(_) => break,
                                        Ok(n) => {
                                            acc.extend_from_slice(&buf[..n]);
                                            while let Some(pos) =
                                                acc.iter().position(|&b| b == b'\n')
                                            {
                                                let line: Vec<u8> = acc.drain(..=pos).collect();
                                                let line = &line[..line.len() - 1];
                                                if let Ok(parsed) =
                                                    serde_json::from_slice::<serde_json::Value>(
                                                        line,
                                                    )
                                                {
                                                    let correlation_id = parsed["correlation_id"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let pattern = parsed["pattern"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let data: Vec<u8> = parsed["data"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .as_bytes()
                                                        .to_vec();

                                                    let cid = correlation_id.clone();
                                                    let context = MessageContext {
                                                        pattern: pattern.clone(),
                                                        correlation_id,
                                                        headers: std::collections::BTreeMap::new(),
                                                    };

                                                    let handler_opt = {
                                                        let h = handlers.lock().unwrap();
                                                        h.get(&pattern).cloned()
                                                    };
                                                    if let Some(handler) = handler_opt {
                                                        let result = handler(data, context).await;
                                                        if let Ok(response) = result {
                                                            if let Ok(msg_bytes) =
                                                                serialize_msg(&cid, &response)
                                                            {
                                                                let _ = writer
                                                                    .write_all(&msg_bytes)
                                                                    .await;
                                                            }
                                                        }
                                                    } else {
                                                        let handler_opt = {
                                                            let h = event_handlers.lock().unwrap();
                                                            h.get(&pattern).cloned()
                                                        };
                                                        if let Some(handler) = handler_opt {
                                                            let _ = handler(data, context).await;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        }
                        Err(_) => break,
                    }
                }
            });

            Ok(())
        })
    }

    fn on_message(&self, pattern: &str, handler: MessageHandler) {
        self.handlers
            .lock()
            .unwrap()
            .insert(pattern.to_string(), handler);
    }

    fn on_event(&self, pattern: &str, handler: EventHandler) {
        self.event_handlers
            .lock()
            .unwrap()
            .insert(pattern.to_string(), handler);
    }

    fn close(&self) -> ServerFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}
