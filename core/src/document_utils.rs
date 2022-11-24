use git2::{Blob, Oid, Tree};
use libipld::{Block, DefaultParams, ipld};
use libipld::IpldCodec::Raw;
use libipld::multihash::Code;
use crate::gpg::Gpg;
use crate::resource::Resource;
use crate::Document;
use crate::errors::Error;

pub struct DocumentUtils;

impl DocumentUtils {
    pub fn commit_update(
        doc: &Document,
        _resource: &Resource,
        update: Vec<u8>,
    ) -> Result<(), git2::Error> {
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

        if doc.ipfs.is_some() {
            let blob_to_return = repo.find_blob(update_oid.clone()).unwrap();
            let oid_blob = update_oid.clone();
            let tree_to_return = update_tree;
            let oid_tree = update_tree_oid;
            let commit_to_return = commit_string;
            let oid_commit = new_signed_commit;

            Self::update_ipfs(doc,blob_to_return,oid_blob,tree_to_return,oid_tree,commit_to_return,oid_commit);
        }

        Ok(())
    }

    pub fn update_ipfs(doc:&Document, blob:Blob, oid_blob:Oid, tree:Tree, oid_tree:Oid, commit:&String, oid_commit:Oid) -> Result<(),Error>{

        // Blob to ipfs store
        let block_blob: ipfs_embed::Block<DefaultParams> =Block::encode(Raw, Code::Blake3_256, &ipld!(blob.content())).unwrap();
        //let cid = block_blob.cid();
        doc.ipfs.as_ref().unwrap().insert(block_blob.clone()).expect("Could not insert block to IPFS store");
        //self.ipfs.unwrap().alias(x, Some(c1.cid()))?; What is this for?

        // Tree to ipfs store
        // the objects it points to as bytes since there is only one blob that might work.
        //tree.iter().next().unwrap().name_bytes();
        let block_tree: ipfs_embed::Block<DefaultParams> =Block::encode(Raw, Code::Blake3_256, &ipld!(tree.iter().next().unwrap().name_bytes())).unwrap();
        doc.ipfs.as_ref().unwrap().insert(block_tree.clone()).expect("Could not insert block to IPFS store");

        // Commit to ipfs store
        /** !!! Is Commit a string??? **/

        let block_commit: ipfs_embed::Block<DefaultParams> =Block::encode(Raw, Code::Blake3_256, &ipld!(commit.as_bytes())).unwrap();
        doc.ipfs.as_ref().unwrap().insert(block_commit.clone()).expect("Could not insert block to IPFS store");




        doc.ipfs.as_ref().unwrap().flush();


        Ok(())
    }
}
