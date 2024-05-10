# Overview

Purpose of this layer is to mitigate Sybil Attack. But also, it is optional as
not all cases require identification.

If we omit this layer then anyone can spawn as many nodes as they like, flood the
bootstrap nodes and create a big network consistent with a lot of malicious nodes.

Though, with right config for extensive gossiping, honest nodes can isolate
malicious nodes overtime. But this approach is far from efficient and not desired
most of the time. Thus, the need for such solution.

ID can be anything other nodes are willing to accept:
- Blockchain Externally Owned Account - if p2p network used as with blockchain.
Collateral is considered to be one of solutions to Sybil Attack. This can
be an option so that there will be less overhead but it's not secure as there
always can be somebody with money.
- OAuth - google, web3auth.io, or any other if it fits you.
