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
