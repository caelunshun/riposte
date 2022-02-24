## Backend service

The backend service in `crates/grpc` serves two roles:
1. User authentication. 
2. Managing multiplayer games. The service keeps a list of currently available
games and proxies connections between clients and the game host. Note that all games
are hosted on client computers; the backend doesn't run any game logic.

### Client to server

Clients communicate with the game server over QUIC on port 19836. 
The protocol uses packets defined in `riposte.proto`. Each packet is sent
with a 32-bit length delimiter over a QUIC stream. Multiple packets may use the same stream. Any number
of streams is allowed; the purpose of using multiple streams is to improve performance
when packets don't need to be received in the same order they were sent.

All QUIC streams should be unidirectional. The backend ignores all bidirectional streams
opened by clients.

Clients can connect to a game server in two ways:
1. Using the backend service as a proxy. The client sends the `JoinGame` gRPC request, which
returns a session ID. It then connects to the backend via QUIC and opens a single stream,
where it sends the session ID as raw bytes with no length delimiter.
The backend then notifies the server of the new connection.
Any streams opened after this point are proxied directly between the client and the server.

2. By directly connecting to the game server. Currently, this is only used for singleplayer games,
where the server is running on localhost.

### Game servers

When configured to run in multiplayer mode, a game server should send the `CreateGame` gRPC request
to the backend. It will receive a session ID in response, which it should keep secret. Next, it should
connect via QUIC to the backend, open one stream, and send the session ID (no length delimiter).

At this point, the game server is ready to accept connections. The backend will add the game to the game list
so clients can join. 

When a new client requests to join the game, the backend opens a new stream and sends a `OpenStream`
packet containing a `NewConnection`. The `NewConnection` contains:
* the user's UUID, which is guaranteed to be authentic
* a connection ID

The server can now open new streams with the client. Since all clients are proxied through the same
QUIC connection, we need a way to associate a stream with a single client. Therefore, the first
data sent on each stream in either direction needs to be the `OpenStream` packet containing the
connection ID of the client using that stream.

## Disconnect handling

If a client disconnects, the backend sends `OpenStream::ClientDisconnected` to the game server.
If the game server disconnects, the backend sends `OpenStream::ConnectionLost` to all clients,
then terminates the connection.
