## ADDED Requirements

### Requirement: TCP transport backend
The framework SHALL provide a TCP socket implementation of `MicroserviceClient` and `MicroserviceServer` using `tokio::net::TcpStream`/`TcpListener`.

#### Scenario: TCP server listens and dispatches
- **WHEN** `TcpServer::listen()` is called on `127.0.0.1:3001`
- **THEN** the server accepts TCP connections, reads length-prefixed messages, deserializes them, dispatches to handlers, and writes responses back

#### Scenario: TCP client connects and sends
- **WHEN** `TcpClient::send("pattern", data)` is called
- **THEN** the client connects to the server, sends the serialized pattern + payload, and reads the response

### Requirement: TCP transport configuration
The framework SHALL provide `TcpClientConfig` and `TcpServerConfig` with host, port, TLS options, and max buffer size.

#### Scenario: TLS encryption
- **WHEN** TLS options are provided in config
- **THEN** the TCP transport uses TLS for encrypted communication
