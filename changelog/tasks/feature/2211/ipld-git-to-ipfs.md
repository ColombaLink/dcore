# git to Interplanetary Linked Data (IPLD)

- Assigned: @timo 

## Problem 

The git objects need to be synchronized with the IPFS bitswap protocol.
Here is an example of how an ipld object is requested: 

[source](https://github.com/ipfs-rust/libp2p-bitswap/blob/9af511a4f41e18eda8611d0b62337228a0c2e948/src/behaviour.rs#L845)
```rust 
    #[async_std::test]
    async fn test_bitswap_get() {
        tracing_try_init();
        let mut peer1 = Peer::new();
        let mut peer2 = Peer::new();
        peer2.add_address(&peer1);

        let block = create_block(ipld!(&b"hello world"[..]));
        peer1.store().insert(*block.cid(), block.data().to_vec());
        let peer1 = peer1.spawn("peer1");

        let id = peer2
            .swarm()
            .behaviour_mut()
            .get(*block.cid(), std::iter::once(peer1));

        assert_complete_ok(peer2.next().await, id);
    }
```

as we can see we need the `cid` of the object and the raw data. 

In terms of a git object we need to transform the git hash to a `cid` and then request the raw data.

## Goal 

we need a function that takes a git hash and returns the multihash (cid). 
The codec code is `0x78` for a raw git object (see [here](https://github.com/multiformats/multicodec/blob/1f8b8f244bca84c764f335fcce5f7073bea163ca/table.csv#L47))

See the [rust-cid test](https://github.com/multiformats/rust-cid/blob/3ea17b63d9f716ace1cb8353a513d0621f39b5d2/tests/lib.rs)
on how to create a cid. 

We also need the inverse function that takes a `cid` and returns the git hash.

## Implementation

- [x] add the latest multihash rust-cid as a dependency https://docs.rs/cid/latest/cid/index.html
- [x] add a new module utils module 'utils.rs' 
- [x] create a function that takes a git hash and returns the multihash (cid)
- [x] create a function that takes a multihash (cid) and returns the git hash
