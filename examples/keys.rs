use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use std::fs::File;
use std::io::{Cursor, Read};
use git2::{BlobWriter, Error, Repository, RepositoryInitOptions};
use std::path::{Path, PathBuf};
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
use openssl::sha::{Sha256, Sha512};
use openssl::x509::X509;
use openssl_sys::DSA;
use pgp::{Deserializable,  SignatureParser, SignedKeyDetails, SignedPublicKey, SignedSecretKey, SignedSecretKeyParser, StandaloneSignature};
use pgp::crypto::HashAlgorithm;
use pgp::types::{Mpi, MpiRef, PublicKeyTrait, SecretKeyTrait, SignedUser};
use pgp::ser::Serialize;
use pgp::types::KeyTrait;

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
    let path = "test/sign-commit/test-repo";
    let mut init_options = RepositoryInitOptions::new();
    // init_options.bare(true);
    let repo = Repository::init_opts(&path, &init_options)?;
    let oid = git2::Oid::from_str("709b01418d301979120fd58916f280cadb28735f")?;
    let commit = repo.find_commit(oid)?;
    {
        let body = commit.message_raw().unwrap();
        let header = commit.raw_header().unwrap();
        println!("{}",header);
        println!("{}",body);
    }

    println!("oid: {}", oid);
    Ok(())
}

fn read_file<P: AsRef<Path> + ::std::fmt::Debug>(path: P) -> File {
    // Open the path in read-only mode, returns `io::Result<File>`
    match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {:?}: {}", path, why),
        Ok(file) => file,
    }
}


fn run() -> Result<(), pgp::errors::Error> {

    let f = read_file(Path::new("./test/sign-commit/public.key"));
    let count = SignedPublicKey::from_armor_single(f);
    // let count = Message::from_bytes(f);


    const DATA :&'static [u8] = b"tree 00b9400c156668f7402ea85760cf2330a50808e4
author Wendy Appleseed <wendy@appleseed.com> 1663942420 +0200
committer Wendy Appleseed <wendy@appleseed.com> 1663942420 +0200
test
";

    const sig :&'static [u8] = b"iHUEABYIAB0WIQRMjoHoCNhpEvQYKFaoucZpnHKj7wUCYy2/FAAKCRCoucZpnHKj7w1AAQDuKjDV5vEhjtXve/29N4kDXDauk1BU39j8CiAPDNoWjQD/fEhYV/xcHkky7Q52WKCquZW2ljhoz3DRhOUfyLi4owg==JXAT";

    {
        // with sha256 digest
        let f = read_file(Path::new("./test/sign-commit/private.key"));
        let count = SignedSecretKey::from_armor_single(f);
        // let count = Message::from_bytes(f);
        for sec in count {
            let passwd_fn = || String::new();

            use sha2::{Sha256, Digest};
            let digest = {
                let mut hasher = Sha256::new();
                hasher.update(DATA);
                hasher.finalize()
            };
            let digest = digest.as_slice();

            let new_sig = sec.0.create_signature(passwd_fn, HashAlgorithm::SHA1, digest)?;
            let verif = sec.0.verify_signature(HashAlgorithm::SHA1, digest, &new_sig);
            println!("..");
        }
    }



    {
            // with sha256 digest
            let f = read_file(Path::new("./test/sign-commit/private.key"));
            let count = SignedSecretKey::from_armor_single(f);
            // let count = Message::from_bytes(f);
            for sec in count {
                let passwd_fn = || String::new();

                let digest = DATA;

                let new_sig = sec.0.create_signature(passwd_fn, HashAlgorithm::SHA1, digest )?;
                let verif = sec.0.verify_signature(HashAlgorithm::SHA1, digest, &new_sig);
                println!("..");
            }
    }



   /*
    for c in count {
        let x = c;
        println!("..");
        let s = [Mpi::from_slice(sig)];
        let v = x.0.verify_signature(HashAlgorithm::SHA1, DATA, &s).as_ref();
        println!("...")
    }


    */



    let can_be_parsed = "-----BEGIN PGP SIGNATURE-----

iHUEABYIAB0WIQS2NbX7Xtnqc/lTXwocxwMQvjkS1QUCWiC+PgAKCRAcxwMQvjkS
1Y5IAQDNk4Bu4sCAHTlvUSS9ioOo9yWDqIvliE1aBvZeZCzDLgEAgQSsQtP8Rqq/
f+SHxLV2cgZpFLcKEIg0odi8Uxv4WAk=
=uBdw
-----END PGP SIGNATURE-----";
    let mut armor = String::from(can_be_parsed);


    let lines: Vec<&str> = armor.lines().collect();
    let header = lines.first();
    {
        print!("{}", header.unwrap());
    }

    let cksum_line = &lines[lines.len() - 2];

    let ok = armor.starts_with("---");
    {

        /* works
        let f = read_file(Path::new("./test/sign-commit/test.sig"));
        let x = StandaloneSignature::from_armor_single(f);
       */

        let x1 = StandaloneSignature::from_string(can_be_parsed);
        // is a valid signature
        // https://cirw.in/gpg-decoder/#-----BEGIN%20PGP%20SIGNATURE-----%20iHUEABYIAB0WIQRMjoHoCNhpEvQYKFaoucZpnHKj7wUCYy2/FAAKCRCoucZpnHKj%207w1AAQDuKjDV5vEhjtXve/29N4kDXDauk1BU39j8CiAPDNoWjQD/fEhYV/xcHkky%207Q52WKCquZW2ljhoz3DRhOUfyLi4owg=%20=JXAT%20-----END%20PGP%20SIGNATURE-----
        let git_commit = "-----BEGIN PGP SIGNATURE-----

 iHUEABYIAB0WIQRMjoHoCNhpEvQYKFaoucZpnHKj7wUCYy2/FAAKCRCoucZpnHKj
 7w1AAQDuKjDV5vEhjtXve/29N4kDXDauk1BU39j8CiAPDNoWjQD/fEhYV/xcHkky
 7Q52WKCquZW2ljhoz3DRhOUfyLi4owg=
 =JXAT
 -----END PGP SIGNATURE-----";

        // can no be parsed
        let x = StandaloneSignature::from_string(git_commit);
        println!("...")
    }


    Ok(())
}






fn main() {
    // load a keypair from a pem file
    // let key_pem = include_bytes!("../test/sign-commit/alice.pem");
    // let cert = X509::from_pem(key_pem).unwrap();
    match sign_git_commit() {
        Ok(_) => println!("success"),
        Err(e) => println!("error: {}", e)
    }
    match run() {
        Ok(_) => println!("success"),
        Err(e) => println!("error: {}", e)
    }


}
