# Protocol

## Framing

Inspired by [WebSockets RFC](https://datatracker.ietf.org/doc/html/rfc6455#section-5.3)
and simplified

- 1 bit - FIN flag - is this the last package
- 3 bit - Reserved - just a padding to skip this byte
- 4 bit - opcode
    - 0000 - `CONTINUATION` - received frame is a continuation of previous unfinished frame
    - 0001 - `CONN_INIT` - init connection with some data
    - 0010 - `CONN_CLOSED` - party disconnected // todo: send in case of graceful shutdown
    - 0011 - `PING` - checking if connection is still alive
    - 0100 - `PONG` - answer if connection is still alive
    - 0101 - `DATA` - frame contains application data
    - 0110 - `NODE_STATUS` - information about other nodes client can connect to.
    - 0111-1111 - reserved for future
- unknown bits - payload. Until received 0 at FIN flag received and no more bits on the network

## Handling Opcodes

### CONTINUATION

Current frame contains data that immediate continuation of the data from
previous frame and new data (i.e. its bytes) should be concatenated to previous
data.

### CONN_INIT

Client shares required information with the server to enable connection.
The info is:
- its server address

### CONN_CLOSED

Sending party is closing the connection. Receiving party should ignore any messages
coming after that (if any) and shut down the TCP stream.

### PING

One party requests second party to reply with `PONG` to prove it is still available.

### PONG

Second party proves it is still available and also returns ping with another Node.
This ping in reply is used by first party to calculate the relative angle of the
nodes.

### DATA

Bytes of data used by an application. Protocol should be ignorant of what these
bytes actually are.

### NODE_STATUS

Party sends information about other nodes in the network.

1. Node #A will skip this step if it already has connected to that node, or it has enough connections
2. Node #B sends to the Node #A `NODE_INFO` with info of another Node #C
3. Node #A also connects to Node #C, go back to 1.
4. With calculated ping from Node #A will calculate position of Node #C

## Message Sequence

### Connecting to another Node

1. Immediately after TCP connection, client on Node #A sends `CONN_INIT` frame
with the required information, measuring the ping. If it is bad, then disconnect.
2. Otherwise, server on Node #B received this frame and saves this information.
3. If Node #B has connects with other nodes, it replies with `NODE_STATUS` frame
4of some other node it is connected to.
4. After successful connection, each node starts sending `PING` frames.

### Message Sequence

After connecting, application sends `DATA` frames.

### Disconnecting

1. Client on Node #A sends `CONN_CLOSED` frame to all the servers
2. Server on Node #A sends `CONN_CLOSED` frame to all the clients

### Receiving `CONN_CLOSED` frame

1. Node #A receives `CONN_CLOSED` frame from Node #B
2. If amount of connected nodes dropped below the threshold:
    1. Connect to the nodes available in `knows_about` metadata.
    2. If that's not enough, send `PONG` frames to known nodes
3. Cleanup info about Node #B.
