use std::collections::HashMap;
use git2::{BlobWriter, Error, Repository, RepositoryInitOptions};
use std::path::{PathBuf};
use crate::gpg::{Gpg, Key};
use crate::resource::Resource;


pub struct Doc {
    pub repository: Repository,
    identity: Key,
    gpg: Gpg,
    resources: HashMap<String, Resource>,
}

unsafe impl Send for Doc {}

pub struct  DocumentInitOptionsIdentity {
    pub fingerprint: String,
    pub public_key: String,
}

pub struct DocumentInitOptions {
    pub directory: PathBuf,
    pub identity: DocumentInitOptionsIdentity,
}

impl Doc {




    pub fn init(args: &DocumentInitOptions) -> Result<Doc, Error> {
        // 1. First we create a bare git repository that build the basis for the dybli document
        let mut data = &mut args.directory.clone();
        data.push(".data");


        let key = &args.directory.clone().push(".keys");
        let mut init_options = RepositoryInitOptions::new();
        init_options.bare(true);
        let repo = Repository::init_opts(&data, &init_options)?;

        let mut resource = Resource::new(String::from("config"));

        let sub = resource.observe_local_transactions(|t, update| {
            println!("Local transaction: ..." );
        });

        resource.local_transaction_subscriptions.insert(sub.id, sub);

        resource.set_resource_meta("config".to_string()).unwrap();

        resource.add_local_update(|mut transaction| {

            let config_root = transaction.get_map("config");

            let public_key = (&args.identity.public_key).to_string();
            let fingerprint = (&args.identity.fingerprint).to_string();
            config_root
                .insert(&mut transaction, fingerprint.as_str().to_owned(),
                        {
                            let mut user_conf = HashMap::new();
                            user_conf.insert("fingerprint".to_owned(), fingerprint.as_str().to_owned());
                            user_conf.insert("publicKey".to_owned(), public_key.as_str().to_owned());
                            user_conf
                        }
                );

            transaction
        }).unwrap();

        let mut resources = HashMap::new();
        resources.insert("config".to_string(), resource);

        Ok(Doc {
            repository: repo,
            identity: Key {
                public: None, // todo: set pk
                fingerprint: args.identity.fingerprint.clone(),
            },
            gpg: Gpg::new(),
            resources
        })
    }

}

#[cfg(test)]
mod tests {
    use std::borrow::BorrowMut;
    use std::collections::HashMap;
    use std::fs;
    use std::ops::Deref;
    use std::path::PathBuf;
    use lib0::any::Any;
    use crate::Doc;
    use crate::resource::Resource;

    #[test]
    fn init_new_doc() {
        let doc_dir = "./.test/doc/init_new_doc/";
        fs::remove_dir_all(doc_dir).ok();
        let mut doc = Doc::init(
            &crate::DocumentInitOptions{
                directory: PathBuf::from(doc_dir),
                identity: crate::doc::DocumentInitOptionsIdentity{
                    fingerprint: String::from("fingerprint"),
                    public_key: String::from("public_key1234"),
                }
            }).unwrap();


        let  r = doc.resources.get("config").unwrap();
        let resource = r.store.transact().get_map("config").to_json();
        let expected = Any::from_json(
            r#"{
              "fingerprint": {
                "fingerprint": "fingerprint",
                "publicKey": "public_key1234"
              }
            }"#,
        )
            .unwrap();
        assert_eq!(resource, expected);
    }

    #[test]
    fn update_resource() {
        let doc_dir = "./.test/doc/init_new_doc/";
        fs::remove_dir_all(doc_dir).ok();
        let mut doc = Doc::init(
            &crate::DocumentInitOptions{
                directory: PathBuf::from(doc_dir),
                identity: crate::doc::DocumentInitOptionsIdentity{
                    fingerprint: String::from("fingerprint"),
                    public_key: String::from("public_key"),
                }
            }).unwrap();


        let  r = doc.resources.get_mut("config").unwrap();
        r.add_local_update(|mut transaction| {

            let config_root = transaction.get_map("config");

            config_root
                .insert(&mut transaction, "fingerprint",
                        {
                            let mut user_conf = HashMap::new();
                            user_conf.insert("fingerprint".to_owned(), "up");
                            user_conf.insert("publicKey".to_owned(), "date");
                            user_conf
                        }
                );

            transaction
        }).unwrap();

        let resource = r.store.transact().get_map("config").to_json();
        let expected = Any::from_json(
            r#"{
              "fingerprint": {
                "fingerprint": "up",
                "publicKey": "date"
              }
            }"#,
        )
            .unwrap();
        assert_eq!(resource, expected);
    }

}
