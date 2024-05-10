# Overview

1. Node #A calculates next id `x`.
2. Node #A sends `DATA` frame with that id and saves it locally.
3. Node #B receives `DATA` frame with id `x`.
4. Node #B broadcasts this exact data further with the same id `x`, excluding
Node #A from broadcast.
5. Node #C does same steps 3-4.
6. Node #A receives `DATA` frame with id `x`.
7. If it is saved locally, ignore this message. Otherwise, pass it to the
application.

## Pseudo-Random Number Generator

P2P requires to use message ids in order to ignore same messages received previously
from other nodes. However, it can happen only when connection between
Nodes #A and #C slower than connection between Nodes #A and #B plus Nodes #B and #C.
Anyway, it is better to handle this.

It was decided that random id suits best. [xoroshiro128**](https://prng.di.unimi.it/xoroshiro128starstar.c)
is used to generate these numbers because no reason to use more sophisticated
number generator.

In theory, we can use small-scale-bit generators to optimize space usage for
two reasons:
- We can reset the state after some time. (but seems risky, TODO)
- We have only a specific relatively small number of nodes, we are connected to.
Our goal not to collide with them, and we can ignore others.
