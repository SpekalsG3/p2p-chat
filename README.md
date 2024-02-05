# Private Chat

This is a p2p chat with asymmetric encryption built in pure rust using
only convenience libraries.

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

## Framing

Inspired by [WebSockets RFC](https://datatracker.ietf.org/doc/html/rfc6455#section-5.3)
and simplified for my use case.

- 1 bit - FIN flag - is this the last package
- 3 bit - Reserved - just a padding to skip this byte 
- 4 bit - opcode
  - 0000 - continuation
  - 0001 - connection closed
  - 0010 - ping - empty payload
  - 0011 - pong - empty payload
  - 0100 - text
  - 0101 - binary - application would first send one msg to inform of binary type with `text` opcode and then send msg(-s) with binary data
  - 0111-1111 - reserved for future
- unknown bits - payload. Until received 0 at FIN flag received and no more bits on the network
