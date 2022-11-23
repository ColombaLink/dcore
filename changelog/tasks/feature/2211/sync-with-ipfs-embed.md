# Sync with IPFS embed

Assigned to: @timo.castelberg, @f.parrillo 

Since using Libp2p direclty and come up with a custom NetowrkBehavior, we can try to use the IPFS embed library. 

This will have a few advantages:

- we do not have to implement our own NetworkBehaviour
- we can use the IPFS functionalities for Peer Discovery, GossipSub, etc. if needed. 


## Tasks 1
- [x] First task get two nodes to connect to each other and exchange a message over GossipSub.
    - finally, in the ipfs_embed lib.rs test section I found what we need!
    - Random note, sled = "0.34.0" looks like an interesting lib.
## Task 2 - integrate ipfs_embed to document 
- [x] Add the ipfs as a property to the document struct 
  - [x] Instantiate the ipfs in the document struct (see sync_ipfs.rs for example)
    - [x] Don't call the listen_on method on the ipfs instance
  - [x] Set gc config to a very high value (not sure if if it interferes otherwise)
  - [x] Test if inserting and retrieving from the ipfs store works (no sync yet, just local)
  - [x] make the commit_update function in document_utils.rs return the oid and the blob 
    - [x] check that nothing breaks with this change 
  - [x] in document.rs: commit_update, call the ipfs store to insert the blob 
    - [x] Build the CID 
    - [x] Create a block
    - [x] Insert the block into the ipfs store
    
### Task 2 - feedback
  - [x] don't use the tmp dir for the ipfs store, use the same dir as the document store (./.data-ipfs)
  - [x] Change the ipfs property to an option value 
  - [x] Add an optional property to the document new constructor argument that holds the ipfs configs used to instantiate ipfs
    - [x] if the ipfs config is not provided, the ipfs property is set to None
# Notes
## Custom store 

We can create a custom store after patching the ipfs_embed lib. 
The store was not able to sync. This is probably because we did not implement all the functions. 

Also, quite some weird type errors occurred. 

The easiest path forward is to fork the ipfs_embed. Then we add a custom store that uses git as a backend.

- **Eventually this is the way to go.**
  - But before we decide we need to run some benchmarks to see how the git backend performs against the sqlite (ipfs_embed) backend.
- **But for now we take the easier route by storing data twice ocne in the git folder and once in the ipfs_embed store.**

```rust 
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use libipld::{Block, Cid, DefaultParams, Ipld};
use anyhow::{Error, Result};
use fnv::FnvHashMap;
use libp2p_bitswap::BitswapStore;
use libipld::{
    codec::References,
    store::{StoreParams},
};

use ipfs_embed::net::NetworkService;
use ipfs_embed::{Executor, ListenerEvent, Multiaddr, NetworkConfig, PeerId, TempPin};
use futures::{stream::Stream, Future};
use ipfs_embed_core::BlockNotFound;

#[derive(Clone, Default)]
struct Store(Arc<Mutex<FnvHashMap<Cid, Vec<u8>>>>);

#[derive(Clone)]
pub struct Ipfs {
    storage: Store,
    network: NetworkService,
}



impl BitswapStore for Store
    {
        type Params = DefaultParams;

    fn contains(&mut self, cid: &Cid) -> Result<bool> {
        Ok(self.0.lock().unwrap().contains_key(cid))
    }

    fn get(&mut self, cid: &Cid) -> Result<Option<Vec<u8>>> {
        Ok(self.0.lock().unwrap().get(cid).cloned())
    }

    fn insert(&mut self, block: &Block<DefaultParams>) -> Result<()> {
        self.0
            .lock()
            .unwrap()
            .insert(*block.cid(), block.data().to_vec());
        Ok(())
    }

    fn missing_blocks(&mut self, cid: &Cid) -> Result<Vec<Cid>> {
        let mut stack = vec![*cid];
        let mut missing = vec![];
        while let Some(cid) = stack.pop() {
            if let Some(data) = self.get(&cid)? {
                let block = Block::<Self::Params>::new_unchecked(cid, data);
                block.references(&mut stack)?;
            } else {
                missing.push(cid);
            }
        }
        Ok(missing)
    }
}



impl Ipfs

{
    pub async fn new() -> Result<Self> {
        let storage =  Store::default();
        let network_config = NetworkConfig::default();
        let network = NetworkService::new(network_config, storage.clone(), Executor::new()).await?;
        Ok(Self { storage, network })
    }

    /// Returns the currently active listener addresses.
    pub fn listeners(&self) -> Vec<Multiaddr> {
        self.network.listeners()
    }

    /// Listens on a new `Multiaddr`.
    pub fn listen_on(&mut self, addr: Multiaddr) -> impl Stream<Item = ListenerEvent> {
        self.network.listen_on(addr)
    }


    /// Inserts a block in to the block store.
    pub fn insert(&self, block: Block<DefaultParams>) -> Result<()> {
        self.storage.0.lock().unwrap().insert(*block.cid(), block.data().to_vec());
        Ok(())
    }

    /// Returns a block from the block store.
    pub fn get(&self, cid: &Cid) -> Result<Block<DefaultParams>> {
        let map = self.storage.0.lock().unwrap();
        let data = map.get(cid);
        match data {
            Some(data) => {
                let block = Block::<DefaultParams>::new_unchecked(*cid, data.to_vec());
                Ok(block)
            },
            None => anyhow::bail!("..."),
        }
    }


    /// Returns the local `PeerId`.
    pub fn local_peer_id(&self) -> PeerId {
        self.network.local_peer_id()
    }


    /// Either returns a block if it's in the block store or tries to retrieve
    /// it from a peer.
    pub async fn fetch(&self, cid: &Cid, providers: Vec<PeerId>) -> Result<Block<DefaultParams>> {
        let store = self.storage.0.lock().unwrap();
        let data = store.get(cid);


        if let Some(data) = data {
            let block = Block::new_unchecked(*cid, data.to_vec());
            return Ok(block);
        }
        if !providers.is_empty() {
            self.network.get(*cid, providers).await?.await?;

            let store = self.storage.0.lock().unwrap();
            let data = store.get(cid);
            if let Some(data) = data {
                let block = Block::new_unchecked(*cid, data.to_vec());
                return Ok(block);
            }
           // tracing::error!("block evicted too soon. use a temp pin to keep the block around.");
        }
        Err(anyhow::anyhow!("block not found"))
    }
}



```
