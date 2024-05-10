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

# Conclusion

It is impossible to verify if another node is honest in 1 to 1 connection with it.
Requires gossip network.
