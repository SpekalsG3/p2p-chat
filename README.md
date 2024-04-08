# Private Chat

NOTE: This is no way production-ready code. It is created for research purposes and
currently stays like that.

---

This is a P2P chat with its own simple protocol ~~and Diffie-Hellman+RSA encryption~~
built with std Rust using only convenience libraries.

# Limitations to pure xterm interface
- If changing cursor horizontally `V100::GoLineUp`/`Down`/`InsertBlankLines`/`MoveWindowUp`,
in the end it has to stay on same line bc otherwise terminal doesn't scroll
  (new lines are outputted but below the last shown line).
- No way to remove the line so when using stdin and submitting the input
via enter then at best can introduce an empty line
- Text wrapping is an issue and now way to solve it pretty
  - Can't **efficiently** know exact dimensions of the terminal at all times
  - Can't know when any character has been inputted

# Protocol

Current protocol is a Peer-to-Peer protocol with self-managing topology,
which means nodes reorganize themselves in the network in the best way
possible without any external influence.

Currently, protocol ignores all security concerns and any network packet loss,
as it's not the main interest of development.
For now, two assumptions are made:
- Each node can fully trust another node
- TCP connection is reliable

## Pseudo-Random Number Generator

P2P nature of a protocol requires to use message ids in order to ignore messages received previously.
It was decided that random id suits best. [xoroshiro128**](https://prng.di.unimi.it/xoroshiro128starstar.c) used to generate these numbers.

In theory, we can use small-scale-bit generators to optimize space usage for two reasons:
- We can reset the state after some time. (Not sure how it helps though xd)
- We have only a specific small number of nodes, we are connected to, we are worried about
not to collide with.

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
  - 0110 - `NODE_INFO` - information about other nodes client can connect to.
  - 0111-1111 - reserved for future
- unknown bits - payload. Until received 0 at FIN flag received and no more bits on the network

## Message Sequence

### Establishing connection

1. Immediately after TCP connection, client on Node #A sends `CONN_INIT` frame with the required information, measuring the ping
2. Server on Node #B saves this info about the Node #A
3. If Node #B has connects with other nodes:
   1. Node #A will skip this step if it already has connected to that node, or it has enough connections
   2. Node #B sends to the Node #A `NODE_INFO` with info of another Node #C
   3. Node #A also connects to Node #C, go back to 1.
   4. With calculated ping from Node #A will calculate position of Node #C
4. Node #B (and Node #C if went through step .3) start sending `PING` frames
expecting `PONG` frame as the response, measuring the ping and
calculating the angle to Node #A

After connecting, application sends `DATA`

### Disconnecting // TODO

1. Client on Node #A sends `CONN_CLOSED` frame to all the servers
2. Servers close stream, do cleanup.
3. Server on Node #A sends `CONN_CLOSED` frame to all the clients
4. Clients disconnect from that server, do cleanup.

### Receiving `CONN_CLOSED` frame // TODO

1. Node #A receives `CONN_CLOSED` frame from Node #B
2. If amount of connected nodes dropped below the threshold:
   1. Connect to the nodes available in `knows_about` metadata.
   2. If that's not enough, do **ping-pong** procedure with known nodes
3. Cleanup info about Node #B.

# Problems
- [ ] Multi-party room
  - [x] How to share messages for all the connected nodes?
  - [ ] How to share node specific messages?
- [ ] Measuring the ping accurately in step 1. TCP is a three-way handshake.
`CONN_INIT` is one way. `PING` is initiated on the server.
`ACK` shouldn't be there. What else?

---

# Notes

## Message id brainstorm

Two u32 ids solution:
- `data_read_id` cannot be node-dependant because each node is likely to be connected
to at least the same two nodes. Which means, these two nodes will then
duplicate this message during the broadcast.
- `data_read_id` cannot be state dependant because if 2 nodes send message
nearly in the same, ids will collide and another node in between will receive only one of them.
- If `data_read_id` is state dependant, it will cause us to miss 1 or 2 first messages
because of the latencies caused by syncing it from different nodes.
- One `u32` can handle 4B messages, so it can be used for really long time.
- Two ids solution seem to be prone to latency issues from the same node. In practice, it should be extremely unlikely case
because node locks state until the whole frame is written to stream, which means
next message will be always written later and thus take longer time.

Random ids solution:
- Should store multiple u32 ids which is a lot less predictable in case of a spam.
However, any node needs to store an id about the message no longer then greatest ping of connected nodes.
This is because the nodes we are connected to will always receive the message from use.
- Random ids are prone to collisions between different nodes

In the end, this id is used like a state of a node itself. Because two clients can send a message
with the same id to a node they are both connected to, it's more accurately to use state
because:
- only author will generate it
- we can sign it to verify it wasn't changed midway
- state changes more randomly because human-based changed in the network

# Security

## Brainstorm the problems

Main problem is P2P network is trust. Node A should trust Node B to share info
from other nodes without modifying it. Without that, cannot trust message contents,
cannot trust P2P network status. However, that's a huge request. Solving three
problems below will allow fearless participation in network.

However, currently, there's no solution to problem 1 or 2 (variants of
an eclipse attack). All existing solutions require mandatory multiple
connections to verify the message with other nodes. It is done either by
public list of IPs or seeders (basically DNS) which share IPs of other nodes.

One possible solution is to use collateral - reputation or some currency.
However, that's prone to dedicated actors to run a long scam or spend as
much money as required until they get what they want.

### 1. Node can filter what to share

- Node won't share other nodes
- Node refuses to broadcast received packages

---

If Node C signs, encrypts or does whatever with
the message and then sends it to Node A through Node B, then Node B can just simply not
send it further. Node A won't see the message.

- Signature or smth of the software used? If Node B has modified the node software,
then Node C can choose to ignore you as untrustworthy.
This signature has to be dynamic and other party has to calculate it for you.
Also, it has to support different builds to support versioning and customization.
  - Node B can run honest private node alongside to generate the signature and use
  them in dishonest software.

### 2. Node can pose as other actors

In the case of group chat, if Node C can read the message sent from Node B,
then it can repackage the message with own signature and send it to Node A.
Node A won't be able to notice that.

Node B can send the message Node C can't read. But that will require a copy
of the message for every other recipient. Not scalable.

Node B can send something that impossible to replace. Not signature, can just replace it.
Recipient has to be unable to read the original message so that's impossible to change it.
But that's the whole point of communication.

### 3. DMs and group chats in single network

Basically solving middle man attack. RSA used to solve that.

Where do you get public key from? One can just send it and if middle man replaced it then
will fait to decrypt so can choose to ignore this node. But that's a lot of trial and error
to ignore all the malicious nodes. But that's the only possibility or otherwise will
require some common server which goes against the whole point of P2P.

That's a similar problem to the first one - if you can verify that package wasn't modified
during it's transportation, then it's not a problem let everyone see encrypted message.
