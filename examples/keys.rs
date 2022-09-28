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
use pgp::{Deserializable, packet, SignatureParser, SignedKeyDetails, SignedPublicKey, SignedSecretKey, SignedSecretKeyParser, StandaloneSignature, types};

use pgp::crypto::{HashAlgorithm, PublicKeyAlgorithm};
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

        let f = read_file(Path::new("./test/sign-commit/test.sig"));
        let x = StandaloneSignature::from_armor_single(f);

        let x1 = StandaloneSignature::from_string(can_be_parsed);
        // is a valid signature
        // https://cirw.in/gpg-decoder/#-----BEGIN%20PGP%20SIGNATURE-----%20iHUEABYIAB0WIQRMjoHoCNhpEvQYKFaoucZpnHKj7wUCYy2/FAAKCRCoucZpnHKj%207w1AAQDuKjDV5vEhjtXve/29N4kDXDauk1BU39j8CiAPDNoWjQD/fEhYV/xcHkky%207Q52WKCquZW2ljhoz3DRhOUfyLi4owg=%20=JXAT%20-----END%20PGP%20SIGNATURE-----
        let git_commit = "-----BEGIN PGP SIGNATURE-----

 iQGzBAABCgAdFiEEawL2OM40EPVbOML0Hu35AZ/dBgEFAmMz4A8ACgkQHu35AZ/d
 BgG3ZwwAlFXVWidS1aE21CTtxJdXsepNUTjsdwF3151z7S6vsmDGPYGQpXxJnzrO
 vPecR8oSFBv2EBf98QigQ4Osw9zclkD14xirK7JkHGDI0/YAT/qxn1N8MLZSH0Ea
 fhdsR6fv1k/9i3lJHnCNOrCP/T6yjznPq1oGO3d6+b5EvZNe8talSRNFeg9zUlSJ
 8uMhG3NdespAyfX9zofoswUfsJUScQnog4FRD/W6OHVd9Bq10S3CYIl0EbP35cSx
 W4IB6qwXz2MrEes/g34ukyw+Y1afd6WpxmHZIU0bvpFcSc2aAIJj6UtH0kRe1wci
 N34yFZHIejxFwGMUEXoWE+mrawxcMgR/3c5Py79GU0Qff4RPEbqsUwAtn1/HBxWL
 yeIy5NnIkksl8+DxpqufF9coj+VjZxHhnkChDHEl3+ALImrlsQaIlsaLbIDsSuly
 mdsgBzKbTZb8X5N1T5b+e5U/idIuCAVm2sqC/bkqbuiufexhjttEOYXojmox8hr2
 B9OMXGYY
 =VBOi
 -----END PGP SIGNATURE-----
";


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

    /*

    match sing_git_commit_by_string() {
        Ok(_) => println!("success"),
        Err(e) => println!("error:...")
    }
     */

    match sign_git_commit() {
        Ok(_) => println!("success"),
        Err(e) => println!("error: {}", e)
    }

}

fn sing_git_commit_by_string() -> Result<(), pgp::errors::Error> {


    const DATA :&'static [u8] = b"tree 00b9400c156668f7402ea85760cf2330a50808e4
author Wendy Appleseed <wendy@appleseed.com> 1663942420 +0200
committer Wendy Appleseed <wendy@appleseed.com> 1663942420 +0200
test
";

    const sig :&'static [u8] = b"iHUEABYIAB0WIQRMjoHoCNhpEvQYKFaoucZpnHKj7wUCYy2/FAAKCRCoucZpnHKj7w1AAQDuKjDV5vEhjtXve/29N4kDXDauk1BU39j8CiAPDNoWjQD/fEhYV/xcHkky7Q52WKCquZW2ljhoz3DRhOUfyLi4owg==JXAT";

    {
        // with sha256 digest
        let f = read_file(Path::new("./test/sign-commit/rsa4096/private.key"));
        let count = SignedSecretKey::from_armor_single(f);

        // let count = Message::from_bytes(f);
        for sec in count {
            let passwd_fn = || String::new();



            use sha1::{Sha1, Digest};
            // create a Sha1 object
            let mut hasher = Sha1::new();

            // process input message
            hasher.update(DATA);
            // acquire hash digest in the form of GenericArray,
            // which in this case is equivalent to [u8; 20]
            let digest = hasher.finalize();

            let new_sig = sec.0.create_signature(passwd_fn, HashAlgorithm::SHA1, &digest[..])?;
            let verif = sec.0.verify_signature(HashAlgorithm::SHA1, &digest[..], &new_sig);
            println!("..");

            match verif {
                Ok(_) => println!("verified signature"),
                Err(e) => println!("signature error")
            }
            let now = chrono::Utc::now();

             let signature = ::pgp::Signature::new(
                 types::Version::Old,
                 packet::SignatureVersion::V4,
                 packet::SignatureType::Binary,
                 PublicKeyAlgorithm::RSA,
                 HashAlgorithm::SHA1,
                 [digest[0], digest[1]],
                 new_sig,
                 vec![
                     packet::Subpacket::SignatureCreationTime(now),
                     packet::Subpacket::Issuer(sec.0.key_id()),
                 ],
                 vec![],
             );

            let standalone_signature = StandaloneSignature::new(signature);
            let armored_signature = standalone_signature.to_armored_string(None);
            let signature_str = match armored_signature {
                Ok(s) => s,
                Err(e) => String::from("error, did not create armored signature string ")
            };
            println!("{}", signature_str);
            println!("Lets now test if we can parse the armored signature");

            {
                let signature  = StandaloneSignature::from_string(&signature_str);
                match signature {
                    Ok(s) => println!("successfully parsed signature"),
                    Err(e) => println!("error... while parsing armored signature ")
                }
            }

            /*
            //! // sign and and write the package (the package written here is NOT rfc4880 compliant)
            //! let mut signature_bytes = Vec::with_capacity(1024);
            //!
            //! let mut buff = Cursor::new(&mut signature_bytes);
            //! packet::write_packet(&mut buff, &signature)
            //!     .expect("Write must succeed");
            //!
            //!
            //! let raw_signature = signature.signature;
            //! verification_key
            //!     .verify_signature(HashAlgorithm::SHA2_256, digest, &raw_signature)
            //!     .expect("Verify must succeed");

             */
        }
    }

    Ok(())

}
