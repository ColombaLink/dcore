



use std::fs::File;

use git2::{Repository, RepositoryInitOptions};
use std::path::{Path};
use gpgme::{Data};

use libp2p::identity::ed25519::Keypair;




use openssl::error::ErrorStack;









use pgp::{Deserializable, packet, SignedPublicKey, SignedSecretKey, StandaloneSignature, types};

use pgp::crypto::{HashAlgorithm, PublicKeyAlgorithm};
use pgp::types::{PublicKeyTrait, SecretKeyTrait};

use pgp::types::KeyTrait;

fn create_keypair() -> Result<(), ErrorStack> {
    println!("test_crypto");

    let kp = Keypair::generate();
    let pk = kp.public();
    let _secret = kp.secret();

    let msg = "hello world".as_bytes();
    let sig = kp.sign(msg);

    let verification =  pk.verify(msg, &sig);
    println!("verification: {}", verification);

    Ok(())
}

fn sign_git_commit() -> Result<(), git2::Error> {
    let path = "test/sign-commit/rsa4096/test-repo";
    let init_options = RepositoryInitOptions::new();
    // init_options.bare(true);
    let repo = Repository::init_opts(&path, &init_options)?;
    let oid = git2::Oid::from_str("ad89fbbb303dda2587b2d729700ae74aa0ebe631")?;

    let commit = repo.find_commit(oid)?;
    {

        println!("From commit");
        println!();
        print!("{}", commit.raw_header().unwrap());
        print!("{}", commit.message_raw().unwrap());
        println!();

        let mut header_string = String::new();

        let mut target_string = String::new();
        use std::fmt::Write;
        writeln!(target_string, "tree {}", commit.header_field_bytes("tree")?.as_str().ok_or_else(|| "").unwrap());
        writeln!(target_string, "parent {}",commit.header_field_bytes("parent")?.as_str().ok_or_else(|| "").unwrap());
        writeln!(target_string, "author {}",commit.header_field_bytes("author")?.as_str().ok_or_else(|| "").unwrap());
        writeln!(target_string, "committer {}",commit.header_field_bytes("committer")?.as_str().ok_or_else(|| "").unwrap());
        header_string = target_string.clone();
        write!(target_string, "{}", commit.message_raw().unwrap());

        println!("Our commit");
        println!("{}", target_string);

        let sig = sing_git_commit(target_string);
        print!("{}", sig.unwrap());

        println!("#######");
        println!("# Lets create a git file, to test if the sig. can be validated by git.");
        println!("#######");


        let mut git_object_string = String::new();
        git_object_string = git_object_string.clone();
        write!(git_object_string, "{}", commit.message_raw().unwrap());

        let git_object_string = String::new();
        print!("{}", commit.raw_header().unwrap());
        print!("{}", commit.message_raw().unwrap());

        use sha1::{Sha1, Digest};
        // create a Sha1 object
        let mut hasher = Sha1::new();
        // process input message
        hasher.update(git_object_string);
        let _digest = hasher.finalize();



        let head = repo.head().expect("reference");
        let oid = head.target().expect("the current tree");
        let head_commit = repo.find_commit(oid).expect("head commit exits");
        let tree = head_commit.tree().expect("head tree exists");

        // todo: update the blobs content, create a commit with the tree builder, create a signed commit.
        let x_tree_entry = tree.get_name("x").unwrap();
        let x_blob = repo.find_blob(x_tree_entry.id())?;
        let new_content = [x_blob.content(), &[10]].concat();
        let x1_blob = repo.blob(&new_content).unwrap();
        let mut builder = repo.treebuilder(Option::Some(&tree)).unwrap();
        builder.insert("", x1_blob, 0o040000);
        let new_tree_oid = builder.write().unwrap();
        let new_tree = repo.find_tree(new_tree_oid).unwrap();
        println!("new tree oid {}", new_tree_oid);


        let authors_signature = git2::Signature::now("Alice", "info@colomba.link").unwrap();

        let commit_buffer = repo.commit_create_buffer(
            &authors_signature,
            &authors_signature,
            "Test commit...",
            &new_tree,
            &[&commit]
        ).unwrap();

        let _commit_string = commit_buffer.as_str();

     //   println!("{}", commit_buffer.as_str().unwrap());
        let commit_signature = gpg_sign_string(&String::from(commit_buffer.as_str().unwrap())).unwrap();
        // println!("{}", commit_signature.unwrap());

        let mut commit_copy = commit_signature.clone();
        let _commit_signature_withoute_new_line =  commit_copy.truncate(commit_copy.len() - 1);;
        let new_signed_commit = repo.commit_signed(
            commit_buffer.as_str().unwrap(),
            commit_copy.as_str(),
            Some("gpgsig")
        ).unwrap();

        repo.reference("refs/heads/main", new_signed_commit, true, "update ref");
    }




    Ok(())
}

fn sing_git_commit(commit: String) ->  Result<String, pgp::errors::Error> {

    let f = read_file(Path::new("./test/sign-commit/rsa4096/private.key"));
    let signed_secret_key = SignedSecretKey::from_armor_single(f);
    let secret_key = signed_secret_key.expect("should load key").0;
    let signature_str;

    let passwd_fn = || String::new();
    use sha1::{Sha1, Digest};
    // create a Sha1 object
    let mut hasher = Sha1::new();

    // process input message
    hasher.update(commit);
    // acquire hash digest in the form of GenericArray,
    // which in this case is equivalent to [u8; 20]
    let digest = hasher.finalize();
    let new_sig = secret_key.create_signature(passwd_fn, HashAlgorithm::SHA1, &digest[..])?;
    let verif = secret_key.verify_signature(HashAlgorithm::SHA1, &digest[..], &new_sig);

    match verif {
        Ok(_) => println!("verified signature"),
        Err(_e) => println!("signature error")
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
            packet::Subpacket::Issuer(secret_key.key_id()),
        ],
        vec![],
    );

    let standalone_signature = StandaloneSignature::new(signature);
    let armored_signature = standalone_signature.to_armored_string(None);
    signature_str = match armored_signature {
        Ok(s) => s,
        Err(_e) => String::from("error, did not create armored signature string ")
    };

    Ok(signature_str)
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
    let _count = SignedPublicKey::from_armor_single(f);
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
            let _verif = sec.0.verify_signature(HashAlgorithm::SHA1, digest, &new_sig);
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
                let _verif = sec.0.verify_signature(HashAlgorithm::SHA1, digest, &new_sig);
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
    let armor = String::from(can_be_parsed);


    let lines: Vec<&str> = armor.lines().collect();
    let header = lines.first();
    {
        print!("{}", header.unwrap());
    }

    let _cksum_line = &lines[lines.len() - 2];

    let _ok = armor.starts_with("---");
    {

        let f = read_file(Path::new("./test/sign-commit/test.sig"));
        let _x = StandaloneSignature::from_armor_single(f);

        let _x1 = StandaloneSignature::from_string(can_be_parsed);
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
        let _x = StandaloneSignature::from_string(git_commit);
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

    /*
    // 6B02F638CE3410F55B38C2F41EEDF9019FDD0601
     let mut ctx = Context::from_protocol(Protocol::OpenPgp).expect("ok");
     let mut keyring = Data::load("./test/sign-commit/rsa4096/private.key").expect("ok");
     for key in ctx.read_keys(&mut keyring).expect("key") {
         println!("{:?}", key);
     }

*/
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
                Err(_e) => println!("signature error")
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
                Err(_e) => String::from("error, did not create armored signature string ")
            };
            println!("{}", signature_str);
            println!("Lets now test if we can parse the armored signature");

            {
                let signature  = StandaloneSignature::from_string(&signature_str);
                match signature {
                    Ok(_s) => println!("successfully parsed signature"),
                    Err(_e) => println!("error... while parsing armored signature ")
                }
            }

        }
    }

    Ok(())

}

pub fn gpg_sign_string(commit: &String) -> Result<String, gpgme::Error> {

    let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
    let file = Path::new("./test/sign-commit/rsa4096/private.key");
    let input = File::open(file).expect("file must exist");
    let mut data = Data::from_seekable_stream(input).expect("get data");
    // mode.map(|m| data.set_encoding(m));
    ctx.import(&mut data)
            .map_err(|e| format!("import failed {:?}", e));

    ctx.set_armor(true);

    let signing_key = "6B02F638CE3410F55B38C2F41EEDF9019FDD0601";

    let key = ctx.get_secret_key(signing_key)?;
    ctx.add_signer(&key)?;
    let mut output = Vec::new();
    let signature = ctx.sign_detached(commit.clone(), &mut output);

    if signature.is_err() {
//        return Err(Error::GPG(signature.unwrap_err()));
        println!("gpg sing error")
    }

    let x = String::from(std::str::from_utf8(&output).unwrap());
    return Ok(x);

}
