use ipfs_embed::multiaddr::multihash::MultihashGeneric;
use git2::Oid;
use ipfs_embed::Cid;

pub fn oid_to_cid(githash: Oid) -> Cid {

    const GIT_CODEC:u64 = 0x78;
    let bytes= githash.as_bytes();
    let append:&[u8] = &[0x11,0x14]; // 11 = sha1 , 14 = length=20 // all hex
    let mut vec=[append,bytes].concat();
    let fin = vec.as_slice();
    let mh = MultihashGeneric::from_bytes(&fin).unwrap();

    Cid::new_v1(GIT_CODEC,mh)
}

pub fn cid_to_oid(cid:Cid)-> Oid{

    let cid_hash = cid.hash(); // should be the Sha2
    let mut dec = cid_hash.digest();
    Oid::from_bytes(dec).unwrap()

}


#[test]
fn test_git_cid() {

    {
        let githash:Oid = Oid::from_str("f6088c584930599a4b7295638a6d11b766b3db98").unwrap();
        let cid = oid_to_cid(githash);
        println!("cid hash {}", cid.to_string());
        let back_to_git = cid_to_oid(cid);
        println!("back to git  {}", back_to_git);
        assert_eq!(githash, back_to_git);

    }
}



