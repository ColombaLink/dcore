use std::collections::HashMap;
use git2::{BlobWriter, Error, Repository, RepositoryInitOptions};
use std::path::{PathBuf};
use libp2p::{identity, PeerId};
use libp2p::identity::ed25519::Keypair;
use openssl::bn::BigNumContext;
use openssl::ec::{EcGroup, EcKey, EcPoint, PointConversionForm};
use openssl::ecdsa::EcdsaSig;
use openssl::encrypt::{Decrypter, Encrypter};
use openssl::error::ErrorStack;

use openssl::sign::{Signer, Verifier};
use openssl::rsa::{Padding, Rsa};
use openssl::pkey::{PKey, Private};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::sha::Sha512;
use openssl::x509::X509;
use openssl_sys::DSA;



fn create_keypair() -> Result<(), ErrorStack> {
    println!("test_crypto");

    let kp = Keypair::generate();
    let pk = kp.public();
    let secret = kp.secret();

    let msg = "hello world".as_bytes();
    let sig = kp.sign(msg);

    let verification =  pk.verify(msg, &sig);
    println!("verification: {}", verification);

    Ok(())
}

fn sign_git_commit() -> Result<(), git2::Error> {
    let path = "./.test/signed_commits";
    let mut init_options = RepositoryInitOptions::new();
    init_options.bare(true);
    let repo = Repository::init_opts(&path, &init_options)?;

    let blob = repo.blob("test".as_bytes())?;
    let mut builder = repo.treebuilder(None)?;
    builder.insert("a", blob, 0o100644).unwrap();
    let tree_oid = builder.write()?;
    let tree = repo.find_tree(tree_oid)?;
    let oid = repo.commit(Some("refs/heads/main"), &repo.signature()?, &repo.signature()?, "test", &tree, &[])?;
    println!("oid: {}", oid);
    Ok(())
}

fn main() {
    // load a keypair from a pem file
    let key_pem = include_bytes!("../test/sign-commit/alice.pem");
    let cert = X509::from_pem(key_pem).unwrap();

    match sign_git_commit() {
        Ok(_) => println!("success"),
        Err(e) => println!("error: {}", e)
    }
}
