# Sync with IPFS embed

Since using Libp2p direclty and come up with a custom NetowrkBehavior, we can try to use the IPFS embed library. 

This will have a few advantages:

- we do not have to implement our own NetworkBehaviour
- we can use the IPFS functionalities for Peer Discovery, GossipSub, etc. if needed. 


## Tasks
- [x] First task get two nodes to connect to each other and exchange a message over GossipSub.
    - finally, in the ipfs_embed lib.rs test section I found what we need!
    - Random note, sled = "0.34.0" looks like an interesting lib.
