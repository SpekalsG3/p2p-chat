# Overview

This is a specialized type of node. It is responsible to share information about
the whole network(s). Each party can setup their own and have their own view of the
network, they are willing to share.

It works by simply accepting listing requests from each node from the network.
Later it will occasionally ping them to check it they still alive.

## Why

This idea greatly reduces risks of an Eclipse attack because each node can check
with that list and connect to any other node.

Also, it is hard to censor. Owners of bootstrap nodes can try to censor who can
read them by IP but there are millions ways to workaround it. With the list, you can
connect to the network from whatever other IP you want.

Not only it's hard to censor users but also hard to censor members of the network.
As each node makes listing requests, it can then easily check if it was added.
If not, such party can then use gossip layer to convince others to reorganize
the network or ignore this bootstrap node.
