# Why

I need some efficient and secure way to store info about the network.
It is required for bootstrap nodes to efficiently probe the network to determine
still alive nodes and for relay nodes themselves to efficiently reorganize each other.

To simplify, any form of stored information about other nodes in the network
is a DHT in some sense.

Currently, I just store everything I know (gathered from pings) in `knows_about` field.
It doesn't feel robust xd

## Kademlia/Chord

Ethereum uses modified form of Kademlia to share a list of nodes.

These DHTs are not quite suitable for my use case because they both focus on message
efficiency in a network of fault-prone nodes. Efficiency is desired but, instead of
defense against fault-prone, I'm interested in defense against malicious actors.
Malicious actors most definitely will be interested in staying online just to temper
with data or something.

## Crawling

Bitcoin uses basic crawling mechanism of asking one node to share info about its
neighbours, connect to them and repeat.

This is exactly what I try to avoid using bootstrap nodes as I've described in
[boostrap node doc](../bootstrap-nodes.md) - malicious node choose to return only
other malicious nodes. For that matter, Bitcoin also implements method using trusted
BIND DNS seeds and static list of stable listening nodes. But that introduces more
trust requirements and doesn't completely solve the problem of crawling.

## SecureRoutingDHT

It is based on reputation, connection redundancy and organization in clusters
(called quorums).

In my setup, reputation will be handled by gossip layer.
Redundancy of some kind will be done anyway. So that's ok.
This approach differs only organization.

## TODO

[StackOverflow guideline](https://stackoverflow.com/questions/28709670/how-do-i-prevent-malicious-dht-clients-that-might-want-to-alter-delete-my-dht-da).
Need to figure out how to use them. 

# Conclusion

In the end, the question is how to organize the network topology.
Which one to choose?
