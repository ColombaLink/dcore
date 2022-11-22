use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use git2::{BranchType, Cred, PushOptions};
use crate::{Document, Identity};
use crate::errors::Error;

pub struct GitSync;

impl GitSync {
    pub(crate) fn clone(doc: Document, remote: &str) -> Result<(), Error> {
        let mut pull_options = git2::FetchOptions::new();
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            Self::get_credentials(&doc.identity)
        });
        pull_options.remote_callbacks(callbacks);


        doc.repository.remote_set_url("origin", remote)?;
        let mut remote = doc.repository.find_remote("origin").unwrap();

        remote.fetch(&["+refs/heads/*:refs/origin/*"], Some(&mut pull_options), None).unwrap();
        Ok(())
    }
}

impl GitSync {
    pub fn sync(doc: Document) -> Result<(), Error> {
        // Frist we need to get the remote repo
        let remote = doc.config_get_remote();
        let remote = match remote {
            Ok(remote) => remote,
            Err(reason) => {
                // todo: add reason to error
                println!("Could not get remote: {}", reason);
                return Err(Error::Other("Could not get remote".to_string()))
            },
        };

        // Then we push all our local event-logs to the remote

        doc.repository.config().unwrap().set_str("user.name", "fuubi").unwrap();
        doc.repository.remote_set_url("origin", remote.as_str())?;

        let local_device = doc.config_get_local_device().unwrap();
        let local_fingerprint = doc.identity.get_fingerprint();
        let identify_push_suffix = format!("{}/{}", local_fingerprint, local_device);

        let mut refs_to_push = HashSet::new();
            doc.repository
            .references()
            .map_err(|e| Error::GitError(e))
            .unwrap()
            .for_each(|log| {
                let reference = log.unwrap().name().unwrap().clone().to_string();
                // refs/local/
                if reference.starts_with("refs/local/") {
                    refs_to_push.insert(reference);
                }
            });

        let mut update_status = HashMap::new();
        let mut remote = doc.repository.find_remote("origin").unwrap();
        for reference in refs_to_push {
            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, allowed_types| {
                Self::get_credentials(&doc.identity)
            });

            callbacks.push_update_reference(|refname, status| {
                update_status.insert(refname.to_string(), status.map(|s| s.to_string()));
                Ok(())
            });
            let mut push_options = PushOptions::new();;
            push_options.remote_callbacks(callbacks);
            let remote_ref = reference.replace("local", "heads");
            remote.push(
                &[format!("{}:{}", reference, remote_ref)],
                Some(&mut push_options)
            ).unwrap();

        }


        let logs_to_pull = doc.repository
            .branches(Some(BranchType::Remote))
            .map_err(|e| Error::GitError(e))
            .unwrap()
            .map(|log| log.unwrap().0 )
            .filter(|log| !log.name().unwrap().unwrap().ends_with(&identify_push_suffix));


        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            Self::get_credentials(&doc.identity)
        });

        let mut pull_options = git2::FetchOptions::new();

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            Self::get_credentials(&doc.identity)
        });
        pull_options.remote_callbacks(callbacks);

        remote.fetch(&["+refs/heads/*:refs/origin/*"], Some(&mut pull_options), None).unwrap();
        // Then we pull all the remote event-logs to our local repo

        Ok(())
    }




    fn get_credentials(identity: &Identity) -> Result<Cred, git2::Error> {
        let public_key = identity.get_armored_public_key().unwrap();
        let private_key = identity.get_armored_private_key().unwrap();

        /*
        let cred = Cred::ssh_key_from_memory(
        "git",
        None,
            &private_key,
            None
        ).unwrap();
         */

        // todo replace wiht ssh key from memory
        let cred = Cred::ssh_key(
            "git",
            None,
            std::path::Path::new("/home/parfab00/.ssh/id_rsa"),
            None).unwrap();

        Ok(cred)
    }

}
#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use std::fs;

    use fs_extra::dir::CopyOptions;
    use std::path::PathBuf;

    use lib0::any::Any;

    use crate::document::DocumentNewOptions;
    use crate::Document;
    use crate::sync_git::GitSync;

    use crate::test_utils::{
        create_test_env, create_test_env_with_new_gpg_key, create_test_env_with_sample_gpg_key,
        create_test_env_with_test_gpg_key, get_test_key,
    };



    #[test]
    fn sync_with_git() {
        let doc_dir = "./.test/sync_git/sync_with_git/";
        create_test_env_with_test_gpg_key(doc_dir.to_string());
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "A84E5D451E9E75B4791556896F45F34A926FBB70".to_string(),
            name: String::from("name"),
            ipfs_config:None,
        }).unwrap();
        let mut doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();

        doc.config_set_remote(&"git@github.com:fuubi/gpgtest.git").unwrap();

        //let content = doc.resources.get("config").unwrap().get_content();
        //assert_eq!(content, "..., just for debugging");

        let remote = doc.config_get_remote().unwrap();
        assert_eq!(remote, "git@github.com:fuubi/gpgtest.git");

        GitSync::sync(doc).unwrap();

        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "A84E5D451E9E75B4791556896F45F34A926FBB70".to_string(),
            name: String::from("name"),
            ipfs_config:None,
        }).unwrap();
        let mut doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();
        doc.load().unwrap();
    }

}
