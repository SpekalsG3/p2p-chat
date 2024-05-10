# P2P Network

A library for bootstrapping secure anonymous p2p networks with focus to minimize
trust.

# Documentation

All documents and high-level descriptions are in [docs folder](docs).

*Documents and code are work in progress.
Documents describe the goal of the network and might not accurately
represent the current status of the protocol. As code evolves, there will be
more additions to the docs.*

# Examples

Examples are available at `/examples` directory. Each one is used for demonstrative
purposes only.

# Protocol



# Problems
- [ ] Multi-party room
  - [x] How to share messages for all the connected nodes?
  - [ ] How to share node specific messages?
- [ ] Measuring the ping accurately in step 1. TCP is a three-way handshake.
`CONN_INIT` is one way. `PING` is initiated on the server.
`ACK` shouldn't be there. What else?
