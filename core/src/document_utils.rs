use std::fs;
use git2::{Blob, Oid, Tree};
use libipld::{Block, DefaultParams, ipld};
use libipld::IpldCodec::Raw;
//use libipld::multihash::Code;
use cid::multihash::Code;
use crate::gpg::Gpg;
use crate::resource::Resource;
use crate::Document;
use crate::errors::Error;
use crate::utils::oid_to_cid;

pub struct DocumentUtils;

impl DocumentUtils {
    pub fn commit_update(
        doc: &Document,
        _resource: &Resource,
        update: Vec<u8>,
    ) -> Result<(Oid,Oid,Oid), git2::Error> {
        let repo = &doc.repository;
        let resource_name = &_resource.name;
        let user_fingerprint = &doc.identity.get_fingerprint();
        let config = doc.repository.config().unwrap().snapshot().unwrap();
        let device = match config.get_str("user.device") {
            Ok(device) => device,
            Err(_) => "device-0",
        };
        let log_name = format!("refs/local/{}/{}/{}", resource_name, user_fingerprint, device);
        let parents = match doc.repository.revparse_ext(log_name.as_str()) {
            Ok((_obj, reference)) => {
                let oid = reference
                    .expect("Found a reference but could not load it.")
                    .target()
                    .expect("Found a reference but could not get its target oid.");
                let head_commit = doc
                    .repository
                    .find_commit(oid)
                    .expect("Found a reference but could not load its commit.");
                Some(head_commit)
            }
            Err(_) => None,
        };

        let update_oid = repo.blob(&update).unwrap();


        let mut builder = repo.treebuilder(None).unwrap();
        builder.insert("update", update_oid, 0o100644).unwrap();
        let update_tree_oid = builder.write().unwrap();
        let update_tree = repo.find_tree(update_tree_oid).unwrap();
        // todo: pass signature info from config
        let authors_signature = git2::Signature::now("Alice", "info@colomba.link").unwrap();

        let commit_buffer = match parents {
            Some(parent) => {
                let commit_buffer = repo
                    .commit_create_buffer(
                        &authors_signature,
                        &authors_signature,
                        "update.",
                        &update_tree,
                        &[&parent],
                    )
                    .unwrap();
                commit_buffer
            }
            None => {
                let commit_buffer = repo
                    .commit_create_buffer(
                        &authors_signature,
                        &authors_signature,
                        "update.",
                        &update_tree,
                        &[],
                    )
                    .unwrap();
                commit_buffer
            }
        };

        let commit_string = &String::from(commit_buffer.as_str().unwrap());
        let commit_identity = &doc.identity;

        // todo: this is not ideal but making the doc mutable just because of this is not nice either
        // look into more details: https://doc.rust-lang.org/error-index.html#E0382
        let mut gpg = Gpg::new();
        let commit_signature = gpg.sign_string(&commit_string, &commit_identity).unwrap();
        let mut commit_copy = commit_signature.clone();
        let _commit_signature_withoute_new_line = commit_copy.truncate(commit_copy.len() - 1);
        let new_signed_commit = repo
            .commit_signed(
                commit_buffer.as_str().unwrap(),
                commit_copy.as_str(),
                Some("gpgsig"),
            )
            .unwrap();

        repo.reference(&log_name, new_signed_commit, true, "update ref")
            .expect("Could not update the reference with the new commit.");


       // values for update_ipfs store

            //let blob_to_return = repo.find_blob(update_oid.clone()).unwrap();
            let oid_blob = update_oid.clone();
            //let tree_to_return = update_tree;
            let oid_tree = update_tree_oid;
            //let commit_to_return = commit_string;
            let oid_commit = new_signed_commit;

           // Self::update_ipfs(doc,blob_to_return,oid_blob,tree_to_return,oid_tree,commit_to_return,oid_commit);


        Ok((oid_blob,oid_tree,oid_commit))
    }

    pub fn update_ipfs(doc:&Document, oid_blob:Oid, oid_tree:Oid,  oid_commit:Oid) -> Result<(),Error>{

        // Get raw git objects from Fs

        let path = doc.repository.path();
        println!("path of repo {}",path.to_str().unwrap());

        let split = oid_blob.to_string();
        let (split_oid1, split_oid2) =split.split_at(2);

        let blob_path = path.join("objects/".to_owned()+split_oid1+"/"+split_oid2);
        let path = blob_path.as_path();
        println!("{}", path.to_str().unwrap());

        let blob_raw= fs::read(path);
        println!("{}", blob_raw.as_ref().unwrap().len());


        //let block_blob: ipfs_embed::Block<DefaultParams> =Block::encode(GIT_CODEC, Code::Sha1, (blob.content())).unwrap();
        let cid = oid_to_cid(oid_blob);
        let block_blob= Block::new(cid,blob_raw.unwrap());

        //Unsupported multihash 17.
        let blocki = block_blob.as_ref();
        println!("{}", blocki.err().unwrap());

        doc.ipfs.as_ref().unwrap().insert(block_blob.unwrap()).expect("Could not insert block to IPFS store");

        // Tree to ipfs store


        let split = oid_tree.to_string();
        let (split_oid1, split_oid2) =split.split_at(2);
        let tree_path = path.join("objects/".to_owned()+split_oid1+"/"+split_oid2);
        let path = tree_path.as_path();
        let tree_raw= fs::read(path);
        let block_tree= Block::new(cid,tree_raw.unwrap());
        doc.ipfs.as_ref().unwrap().insert(block_tree.unwrap()).expect("Could not insert block to IPFS store");

        // Commit to ipfs store

        let split = oid_tree.to_string();
        let (split_oid1, split_oid2) =split.split_at(2);
        let commit_path = path.join("objects/".to_owned()+split_oid1+"/"+split_oid2);
        let path = commit_path.as_path();
        let commit_raw= fs::read(path);
        let block_commit= Block::new(cid,commit_raw.unwrap());
        doc.ipfs.as_ref().unwrap().insert(block_commit.unwrap()).expect("Could not insert block to IPFS store");

        doc.ipfs.as_ref().unwrap().flush();


        Ok(())
    }
}
