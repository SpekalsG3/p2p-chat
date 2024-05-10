# Overview

It is a Peer-to-Peer network. It consists of three key layers:
1. Bootstrap Node - solves Eclipse attack by providing a sufficient number of nodes
for the network.
2. Identity layer - part of a solution for Sybil attack by reducing possibilities of
spawning multiple nodes by one person.
3. Gossip Layer - part of a solution to Sybil attack and other malicious actors by
making it significantly harder to act dishonestly (censoring, editing data, etc.)

Such structure originates from simple yet challenging though-process:
1. It's impossible to solve data editing or censorship in 1 to 1 connection with 
malicious node. Currently, the only way to mitigate this is gossip networks.
2. To construct gossip network need a lot of connections. In practice, people use
bootstrap nodes or trusted nodes. Personally, I want to minimize the need of trust.
3. In order for bootstrap nodes to make sense, need some IDs for nodes to mitigate
sybil attack. Some kind of identification solution.
This three key layers should allow for secure anonymous p2p layer.

Using bootstrap nodes, everybody can get enough information to connect to the
network without trusting someone exclusively. It doesn't matter if bootstrap nodes
censor someone as each node can choose to reorganize itself later (e.g. change IP).

To participate in the network, newcomer first goes through identity layer and receives
some personalized ID: DHT route, OAuth token from third-party services or
none at all - anything other nodes willing to accept.
