use std::collections::HashMap;
use git2::{BlobWriter, Error, Repository, RepositoryInitOptions};
use std::path::{PathBuf};
use libp2p::{identity, PeerId};
use libp2p::identity::ed25519::Keypair;
use openssl::bn::BigNumContext;
use openssl::ec::{EcGroup, EcKey, EcPoint, PointConversionForm};
use openssl::encrypt::{Decrypter, Encrypter};
use openssl::error::ErrorStack;

use openssl::sign::{Signer, Verifier};
use openssl::rsa::{Padding, Rsa};
use openssl::pkey::{PKey, Private};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::sha::Sha512;

pub struct Doc {
    repository: Repository
}

unsafe impl Send for Doc {}

pub struct DocumentInitOptions {
    pub directory: PathBuf,
}

impl Doc {

    pub fn init(args: &DocumentInitOptions) -> Result<(), Error> {
        // 1. First we create a bare git repository that build the basis for the dybli document
        let path = &args.directory;
        let mut init_options = RepositoryInitOptions::new();
        init_options.bare(true);
        let repo = Repository::init_opts(&path, &init_options)?;

        // 2. next we create the config
        //      for this we have to init a yrs document as the config resource is a crdt
        //
        //      sample config:
        //          { "publicKey": { "alias": "alice" } }

        let config_resource = yrs::Doc::new();
        let mut config_transaction = config_resource.transact();
        let config_root = config_transaction.get_map("config");

        config_root
            .insert(&mut config_transaction, "publicKey".to_owned(),
                    {
                        let mut user_conf = HashMap::new();
                        user_conf.insert("alias".to_owned(), "alice");
                        user_conf
                    }
        );

        let config = config_transaction.encode_update_v2();
        let id = repo.blob(&config).unwrap();
        println!("{}", id);

        Ok(())
    }
}
