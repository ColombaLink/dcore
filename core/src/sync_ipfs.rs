

mod test {
    use std::time::Duration;
    use futures::prelude::*;
    use ipfs_embed::{Config, Ipfs, Multiaddr, NetworkConfig, PeerId, StorageConfig};
    use ipfs_embed::identity::ed25519::Keypair;
    use libipld::{store::StoreParams, Cid, IpldCodec, Block, Ipld};
    use tempdir::TempDir;
    use crate::errors::Error;

    use libipld::{
        alias, cbor::DagCborCodec, ipld, multihash::Code, raw::RawCodec, store::DefaultParams,
    };
    #[derive(Debug, Clone)]
    struct Sp;

    impl StoreParams for Sp {
        type Hashes = libipld::multihash::Code;
        type Codecs = IpldCodec;
        const MAX_BLOCK_SIZE: usize = 1024 * 1024 * 4;
    }

    fn tracing_try_init() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init()
    }


    async fn create_store(enable_mdns: bool) -> ipfs_embed_core::Result<(Ipfs<DefaultParams>, TempDir)> {
        let tmp = TempDir::new("ipfs-embed")?;
        let sweep_interval = Duration::from_millis(10000);
        let storage = StorageConfig::new(None, None, 10, sweep_interval);

        let mut network = NetworkConfig::new(Keypair::generate());
        if !enable_mdns {
            network.mdns = None;
        }

        let mut ipfs = Ipfs::new(Config { storage, network }).await?;
        ipfs.listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
            .next()
            .await
            .unwrap();
        Ok((ipfs, tmp))
    }

    fn create_block(bytes: &[u8]) -> ipfs_embed_core::Result<Block<DefaultParams>> { Block::encode(RawCodec, Code::Blake3_256, bytes) }

    #[async_std::test]
    async fn test_local_store() -> ipfs_embed_core::Result<()> {
        tracing_try_init();
        let (store, _tmp) = create_store(false).await?;
        let block = create_block(b"test_local_store")?;
        let mut tmp = store.create_temp_pin()?;
        store.temp_pin(&mut tmp, block.cid())?;
        let _ = store.insert(block.clone())?;
        let block2 = store.get(block.cid())?;
        assert_eq!(block.data(), block2.data());
        Ok(())
    }


    #[async_std::test]
    #[ignore] // test is too unreliable for ci
    async fn test_exchange_mdns() -> ipfs_embed_core::Result<()> {
        tracing_try_init();
        let (store1, _tmp) = create_store(true).await?;
        let (store2, _tmp) = create_store(true).await?;
        let block = create_block(b"test_exchange_mdns")?;
        let mut tmp1 = store1.create_temp_pin()?;
        store1.temp_pin(&mut tmp1, block.cid())?;
        let _ = store1.insert(block.clone())?;
        store1.flush().await?;
        let mut tmp2 = store2.create_temp_pin()?;
        store2.temp_pin(&mut tmp2, block.cid())?;
        let block2 = store2
            .fetch(block.cid(), vec![store1.local_peer_id()])
            .await?;
        assert_eq!(block.data(), block2.data());
        Ok(())
    }

    macro_rules! assert_unpinned {
        ($store:expr, $block:expr) => {
            assert_eq!(
                $store
                    .reverse_alias($block.cid())
                    .unwrap()
                    .map(|a| !a.is_empty()),
                Some(false)
            );
        };
    }
    macro_rules! assert_pinned {
        ($store:expr, $block:expr) => {
            assert_eq!(
                $store
                    .reverse_alias($block.cid())
                    .unwrap()
                    .map(|a| !a.is_empty()),
                Some(true)
            );
        };
    }

    fn create_ipld_block(ipld: &Ipld) -> ipfs_embed_core::Result<Block<DefaultParams>> {
        Block::encode(DagCborCodec, Code::Blake3_256, ipld)
    }
    #[async_std::test]
    async fn test_sync() -> ipfs_embed_core::Result<()> {
        tracing_try_init();
        let (mut local1, _tmp) = create_store(false).await?;
        let (mut local2, _tmp) = create_store(false).await?;
        local1.add_address(local2.local_peer_id(), local2.listeners()[0].clone());
        local2.add_address(local1.local_peer_id(), local1.listeners()[0].clone());

        let a1 = create_ipld_block(&ipld!({ "a": 0 }))?;
        let b1 = create_ipld_block(&ipld!({ "b": 0 }))?;
        let c1 = create_ipld_block(&ipld!({ "c": [a1.cid(), b1.cid()] }))?;
        let b2 = create_ipld_block(&ipld!({ "b": 1 }))?;
        let c2 = create_ipld_block(&ipld!({ "c": [a1.cid(), b2.cid()] }))?;
        let x = alias!(x);

        let _ = local1.insert(a1.clone())?;
        let _ = local1.insert(b1.clone())?;
        let _ = local1.insert(c1.clone())?;
        local1.alias(x, Some(c1.cid()))?;
        local1.flush().await?;
        assert_pinned!(&local1, &a1);
        assert_pinned!(&local1, &b1);
        assert_pinned!(&local1, &c1);

        local2.alias(&x, Some(c1.cid()))?;
        local2
            .sync(c1.cid(), vec![local1.local_peer_id()])
            .await?
            .await?;
        local2.flush().await?;
        assert_pinned!(&local2, &a1);
        assert_pinned!(&local2, &b1);
        assert_pinned!(&local2, &c1);

        let _ = local2.insert(b2.clone())?;
        let _ = local2.insert(c2.clone())?;
        local2.alias(x, Some(c2.cid()))?;
        local2.flush().await?;
        assert_pinned!(&local2, &a1);
        assert_unpinned!(&local2, &b1);
        assert_unpinned!(&local2, &c1);
        assert_pinned!(&local2, &b2);
        assert_pinned!(&local2, &c2);

        local1.alias(x, Some(c2.cid()))?;
        local1
            .sync(c2.cid(), vec![local2.local_peer_id()])
            .await?
            .await?;
        local1.flush().await?;
        assert_pinned!(&local1, &a1);
        assert_unpinned!(&local1, &b1);
        assert_unpinned!(&local1, &c1);
        assert_pinned!(&local1, &b2);
        assert_pinned!(&local1, &c2);

        local2.alias(x, None)?;
        local2.flush().await?;
        assert_unpinned!(&local2, &a1);
        assert_unpinned!(&local2, &b1);
        assert_unpinned!(&local2, &c1);
        assert_unpinned!(&local2, &b2);
        assert_unpinned!(&local2, &c2);

        local1.alias(x, None)?;
        local2.flush().await?;
        assert_unpinned!(&local1, &a1);
        assert_unpinned!(&local1, &b1);
        assert_unpinned!(&local1, &c1);
        assert_unpinned!(&local1, &b2);
        assert_unpinned!(&local1, &c2);
        Ok(())
    }
}

