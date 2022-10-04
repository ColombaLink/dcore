use std::borrow::Borrow;
use crate::{Document, Identity};
use std::fmt::Write;
use git2::Repository;
use gpgme::HashAlgorithm;
use pgp::{Deserializable, packet, SignatureParser, SignedKeyDetails, SignedPublicKey, SignedSecretKey, SignedSecretKeyParser, StandaloneSignature, types};
use pgp::crypto::PublicKeyAlgorithm;
use pgp::types::{KeyTrait, PublicKeyTrait, SecretKeyTrait};
use crate::gpg::Gpg;
use crate::resource::Resource;

struct DocumentUtils;

impl DocumentUtils {

    fn commit_update(mut doc: Document, resource: &Resource, update: Vec<u8>) -> Result<(), git2::Error> {
        let repo = &doc.repository;
        let resource_name = "config";
        let user_public_key = "public_key";
        let (obj, reference) = doc.repository.revparse_ext(user_public_key).unwrap();
        let oid =   reference.unwrap().target().unwrap();

        let head_commit = doc.repository.find_commit(oid)?;
        let tree = head_commit.tree().expect("head tree exists");

        let update_oid = repo.blob(&update).unwrap();
        let mut builder = repo.treebuilder(None).unwrap();
        builder.insert("update", update_oid, 0o100644).unwrap();
        let update_tree_oid = builder.write().unwrap();
        let update_tree = repo.find_tree(update_tree_oid).unwrap();
        let authors_signature = git2::Signature::now("Alice", "info@colomba.link").unwrap();


        let commit_buffer = repo.commit_create_buffer(
            &authors_signature,
            &authors_signature,
            "Test commit...",
            &update_tree,
            &[&head_commit]
        ).unwrap();


        let commit_string = &String::from(commit_buffer.as_str().unwrap());
        let commit_identity = doc.identity;
        let commit_signature = doc.gpg.sign_string(&commit_string, &commit_identity).unwrap();
        let mut commit_copy = commit_signature.clone();
        let commit_signature_withoute_new_line =  commit_copy.truncate(commit_copy.len() - 1);;
        let new_signed_commit = repo.commit_signed(
            commit_buffer.as_str().unwrap(),
            commit_copy.as_str(),
            Some("gpgsig")
        ).unwrap();

        repo.reference("refs/heads/main", new_signed_commit, true, "update ref");

        Ok(())
    }


}
