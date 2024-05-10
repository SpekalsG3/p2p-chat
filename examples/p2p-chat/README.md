# p2p-xterm-chat

Note: it is implemented using Xterm escape codes so be sure they are supported.

To run example application, simply run the following command.
It will run node server at `127.0.0.1:6971` and will connect to `127.0.0.1:6969`.

```bash
cargo r -- -s 127.0.0.1:6971 -c 127.0.0.1:6969
```

If you don't have any server to connect to, simply omit `-c` like that:

```bash
cargo r -- -s 127.0.0.1:6969
```

With more than one node in the network, it will be fully functional simple chat.

## Limitations to pure xterm interface

- If changing cursor horizontally `V100::GoLineUp`/`Down`/`InsertBlankLines`/`MoveWindowUp`,
  in the end it has to stay on same line bc otherwise terminal won't scroll
  (new lines are outputted but below the last shown line).
- No way to remove the line
- Text wrapping is an issue and idk pretty solution
    - Can't **efficiently** know exact dimensions of the terminal at all times
    - Can't know when any character has been inputted
