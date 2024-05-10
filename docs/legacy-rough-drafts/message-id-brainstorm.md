## Message id brainstorm

Two u32 ids solution:
- `data_read_id` cannot be node-dependant because each node is likely to be
connected to at least the same two nodes. Which means, these two nodes will then
duplicate this message during the broadcast.
- `data_read_id` cannot be state dependant because if 2 nodes send message
nearly in the same, ids will collide and another node in between will receive only
one of them.
- If `data_read_id` is state dependant, it will cause us to miss 1 or 2 first
messages because of the latencies caused by syncing it from different nodes.
- One `u32` can handle 4B messages, so it can be used for really long time.
- Two ids solution seem to be prone to latency issues from the same node.
In practice, it should be extremely unlikely case because node locks state until
the whole frame is written to stream, which means next message will be always
written later and thus take longer time.

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
