# Problem

While users would need only a small N of all nodes, I'm not sure how
to provide them securely.

Malicious bootstrap node can control what addresses to return. Thus,
users cannot trust the results without some kind of proof. But I'm not sure
if it worth it in terms of performance.

Bitcoin node has 20k nodes. Ethereum uses 158 bytes for bootnode address.
This would result in 3MB of data to transfer each time.

On the other hand, with proof it would require two requests without a chance
to cache values. While it is easy to proof that indices/keys were selected randomly,
it is harder to proof that address really connected to these indices/keys.
This would require first to share polynomial commitment and then do cryptography
calculations N times per request. And commitment would change each time there's
a change in the network (node went offline, new node connected, etc.).
