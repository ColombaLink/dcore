use std::collections::HashMap;
use std::path::PathBuf;

use git2::{BranchType, Repository, RepositoryInitOptions};
use yrs::{Map, PrelimMap, Transaction, Update};
use yrs::types::Value;
use yrs::updates::decoder::Decode;

use crate::document_utils::DocumentUtils;
use crate::errors::Error;
use crate::gpg::{Gpg, Key};
use crate::Identity;
use crate::resource::Resource;

pub struct Document {
    pub name: String,
    pub repository: Repository,
    pub identity: Identity,
    pub gpg: Gpg,
    pub resources: HashMap<String, Resource>,
}

unsafe impl Send for Document {}

pub struct  DocumentInitOptionsIdentity {
    pub fingerprint: String
}

pub struct DocumentInitOptions {
    pub directory: PathBuf,
    pub identity: DocumentInitOptionsIdentity,
}

pub struct DocumentNewOptions {
    pub directory: PathBuf,
    pub name: String,
    pub identity_fingerprint: String
}

impl Document {

    pub fn new(options: DocumentNewOptions) -> Result<Document, Error> {
        let data_dir = PathBuf::from(options.directory).join("./.data");
        let repository = Repository::init_opts(&data_dir, &RepositoryInitOptions::new().bare(true)).map_err(|e| Error::GitError(e))?;
        let mut gpg = Gpg::new();
        let identity = Identity::from_fingerprint(&mut gpg, &options.identity_fingerprint)
            .expect(("Could not find the identity with the provided fingerprint ".to_string() + &options.identity_fingerprint).as_str());

        return Ok(Document {
            name: options.name,
            repository,
            identity,
            gpg,
            resources: HashMap::new(),
        });
    }

    /// Frist call Document::new(...) then doc.init() to create the config resource
    pub fn init(self, fingerprint: &String, public_key: &String) -> Result<Document, Error> {
        if self.resources.contains_key("config") {
            return Err(Error::Other("Document already initialized because the config resource exists".to_string()));
        }
        let mut resource = Resource::new(String::from("config"));

        let sub = resource.observe_local_transactions(|_t, _update| {
            println!("Local transaction: ..." );
        });

        self.identity.get_fingerprint();

        resource.local_transaction_subscriptions.insert(sub.id, sub);

        let update = resource.set_resource_meta("config".to_string()).unwrap();
        let _ = &self.commit_update(&update, &resource);

        let update = resource.add_local_update(|mut transaction| {

            let config_root = transaction.get_map("config");

            let public_key = public_key.clone();
            let fingerprint = fingerprint.clone();
            config_root.insert(&mut transaction, fingerprint.as_str().to_owned(), PrelimMap::<i32>::from(HashMap::default()));
            let id_map = config_root.get(fingerprint.as_str().to_owned().as_str()).unwrap().to_ymap().unwrap();
            id_map.insert(&mut transaction, "fingerprint".to_string(),fingerprint.as_str().to_owned() );
            id_map.insert(&mut transaction, "public_key".to_string(),public_key.as_str().to_owned() );
            id_map.insert(&mut transaction, "alias".to_string(),fingerprint.as_str().to_owned() );

            transaction
        }).unwrap();


        let _ = &self.commit_update(&update, &resource);

        let mut resources = HashMap::new();
        resources.insert("config".to_string(), resource);

        Ok(Document {
            name: "name".to_string(),
            repository: self.repository,
            identity: self.identity,
            gpg: Gpg::new(),
            resources
        })
    }

    fn commit_update(&self, update: &Vec<u8>, resource: &Resource) {
        DocumentUtils::commit_update(&self, resource, update.to_owned()).expect("TODO: panic message");
    }


    pub fn load(&mut self) -> Result<(), Error> {


       let config_logs_head_oids =  self.repository.branches(Some(BranchType::Local)).map_err(|e| Error::GitError(e)).unwrap().filter(|branch| {
           let log = &branch.as_ref().unwrap().0;
           let log_name = log.name().unwrap().unwrap().clone();
           log_name.starts_with("config/")
       }).map(|log|  {
           let log = &log.as_ref().unwrap().0;
           let oid = log.get().target().unwrap().to_owned();
           oid
       });

        let resource = Resource::new(String::from("config"));
        let revwalk =  &mut self.repository.revwalk().map_err(|e| Error::GitError(e)).unwrap();
        revwalk.set_sorting(git2::Sort::REVERSE).map_err(|e| Error::GitError(e)).unwrap();
        let mut t =   resource.store.transact();
        for  oid in config_logs_head_oids {
            revwalk.push(oid).map_err(|e| Error::GitError(e)).unwrap();
            println!("log: {}", oid.to_string());
            let updates = revwalk
                .flat_map(|id| self.repository.find_commit(id.unwrap()))
                .map(|commit| commit.tree().unwrap())
                .map(|tree| tree.get_name("update").unwrap().to_object(&self.repository).unwrap())
                .map(|object| object.peel_to_blob().unwrap())
                .map(|blob| blob.content().to_vec())
                .map(|content| Update::decode_v2(content.as_slice()).unwrap());

            let update =  Update::merge_updates(updates);
            t.apply_update(update);
        }

        t.commit();
        self.resources.insert("config".to_string(), resource);

        Ok(())
    }

    /*
fn buildNext<'a>(transaction: &'a mut Transaction, current_map: &'a mut Map, key_parts: &'a mut dyn Iterator<Item=&str>, value: &'a str) -> &mut Map {                                       &mut Map {
    let key = key_parts.next().unwrap();
}
    let next_key = key_parts.next().unwrap();
    let next_map = current_map.get_map(next_key);
    if key_parts.next().is_some() {
    return buildNext(transaction, &mut next_map, key_parts);
    }
    let key = key_parts.next().unwrap();
    current_map.insert(transaction, key.to_owned(), value.to_owned());
    next_map
*/

/*
    fn build_map<'a>(transaction: &'a mut Transaction, current_map: &'a mut Map,  key_parts: &'a mut dyn Iterator<Item=&str>, value: &'a str) -> Map {
        let key = key_parts.next().unwrap();

        match key_parts.peekable().peek() {
            Some(_) => {
                let mut next_map = current_map.get(key);
                match next_map {
                    Some(next_value) => {
                        // todo: this could also not be a map
                        let mut next_map = next_value.to_ymap().unwrap();
                        let next_value_map = Document::build_map(transaction, &mut next_map, key_parts, value);

                        next_map.insert(transaction, key.to_owned(), next_value_map.to_owned()).unwrap();
                        next_map
                    },
                    None => {
                        let mut next_map = HashMap::new();
                        let m = Self::build_map(transaction, &mut next_map, key_parts, value);
                        next_map.insert(key.to_owned(), m.to_owned());
                        next_map
                    }
                }
            },
            None => {
                let mut map = HashMap::new();
                map.insert( key.to_owned(), value.to_owned()).unwrap();
                map
            }
        }

    }

 */
        /// the key can either be the root or a sub key that are separated by a dot
    /// e.g. key = "config.users.fingerprint", value = "1234"
    /// { "config" : { "users" : { "fingerprint" : "1234" } } }
    pub fn update_resource_with_key_value(&mut self, resource_name: &str, key: &str, value: &str) -> Result<(), Error> {


        let resource = self.resources.get_mut(resource_name).unwrap();
        let update = resource.add_local_update(|mut transaction| {

            let mut key_parts = key.split(".");
            let root_key = key_parts.next().unwrap();
            let root_map = transaction.get_map(resource_name);
            let mut current_map = root_map.clone();

            while let Some(key) = key_parts.next() {
                if key_parts.clone().peekable().peek().is_some() {
                    // there will be a next key
                    // check if the current key already exists
                    match current_map.get(key).unwrap().to_ymap() {
                        Some(map) => current_map = map.clone(),
                        None => {
                            // this does not work correctly at the moment: it overwrites the the value...
                            // todo: fix nested key
                            let next_map:HashMap<String, String> = HashMap::new();
                            current_map.insert(&mut transaction, key.to_owned(), next_map);
                            current_map = current_map.get(key).unwrap().to_ymap().unwrap().clone();
                        }
                    }
                } else {
                    // last key, so we reached the root
                    current_map.insert(&mut transaction, key.to_owned(), value.to_owned());
                }
            }

            transaction
        }).unwrap();

        let resource1 = self.resources.get(resource_name).unwrap();
        self.commit_update(&update, &resource1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;


    use std::path::PathBuf;

    use lib0::any::Any;

    use crate::{Document};
    use crate::document::DocumentNewOptions;


    use crate::test_utils::{create_test_env, create_test_env_with_new_gpg_key, create_test_env_with_sample_gpg_key, get_test_key};

    #[test]
    fn new_doc() {
        let doc_dir = "./.test/doc/new_doc/";
        create_test_env(doc_dir.to_string());

        let _doc = Document::new(
            DocumentNewOptions {
                directory: PathBuf::from(doc_dir),
                identity_fingerprint: get_test_key().fingerprint,
                name: String::from("name"),
            }).unwrap();
    }


    #[test]
    fn init_new_doc() {
        let doc_dir = "./.test/doc/init_new_doc/";
        let (_dir, key) = create_test_env_with_sample_gpg_key(doc_dir.to_string());

        let doc = Document::new(
            DocumentNewOptions {
                directory: PathBuf::from(doc_dir),
                identity_fingerprint: key.fingerprint.clone(),
                name: String::from("test-doc1"),
            }).unwrap();

        let doc = doc.init(&get_test_key().fingerprint, &get_test_key().public_key).unwrap();

        let  r = doc.resources.get("config").unwrap();
        let resource = r.store.transact().get_map("config").to_json();

        let expected = Any::from_json(
            r#"{
              "39069565EA65A07AE1FBB4BB9B484B5D677BC2EA": {
                "fingerprint": "39069565EA65A07AE1FBB4BB9B484B5D677BC2EA",
                "publicKey": "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\nmDMEYzxVCxYJKwYBBAHaRw8BAQdAIBFXz9lWTbRUZk8QdbtZIDzT8EksDBLUrD5I\no4wKjQi0GkFsaWNlIDxhbGljZUBjb2xvbWJhLmxpbms+iJAEExYIADgWIQQ5BpVl\n6mWgeuH7tLubSEtdZ3vC6gUCYzxVCwIbAwULCQgHAgYVCgkICwIEFgIDAQIeAQIX\ngAAKCRCbSEtdZ3vC6hwcAP9sPv78aC+t4MCasarWYv9FMtJ3aZMgpZchCCJD0b49\nowEA9DSYX43Sf2btvmjjTRvmjSDdG/CzZ11/FZwCbRlJXws=\n=JSAK\n-----END PGP PUBLIC KEY BLOCK-----"
              }
            }"#,
        )
            .unwrap();
        assert_eq!(resource, expected);
    }



    #[test]
    fn load_doc() {
        let doc_dir = "./.test/doc/load_doc/";
        let (_dir, key) = create_test_env_with_new_gpg_key(doc_dir.to_string());

        let doc = Document::new(
            DocumentNewOptions {
                directory: PathBuf::from(doc_dir),
                identity_fingerprint: key.fingerprint.clone(),
                name: String::from("test-doc1"),
            }).unwrap();

        let doc = &mut doc.init(&key.fingerprint, &get_test_key().public_key).unwrap();


        let doc_to_load =&mut Document::new(
            DocumentNewOptions {
                directory: PathBuf::from(doc_dir),
                identity_fingerprint: key.fingerprint.clone(),
                name: String::from("test-doc1"),
            }).unwrap();

        doc_to_load.load().unwrap();

        let  r = doc_to_load.resources.get("config").unwrap();
        let resource = r.store.transact().get_map("config").to_json();

        let  r = doc.resources.get("config").unwrap();
        let expected = r.store.transact().get_map("config").to_json();

        assert_eq!(resource, expected);

    }


    #[test]
    fn update_resource() {
        let doc_dir = "./.test/doc/init_new_doc/";
        create_test_env_with_sample_gpg_key(doc_dir.to_string());
        let doc = Document::new(
            DocumentNewOptions {
                directory: PathBuf::from(doc_dir),
                identity_fingerprint: get_test_key().fingerprint,
                name: String::from("name"),
            }).unwrap();

        let mut doc = doc.init(&get_test_key().fingerprint, &get_test_key().public_key).unwrap();


        let  r = doc.resources.get_mut("config").unwrap();
        r.add_local_update(|mut transaction| {

            let config_root = transaction.get_map("config");

            config_root
                .insert(&mut transaction, "39069565EA65A07AE1FBB4BB9B484B5D677BC2EA",
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
              "39069565EA65A07AE1FBB4BB9B484B5D677BC2EA": {
                "fingerprint": "up",
                "publicKey": "date"
              }
            }"#,
        )
            .unwrap();
        assert_eq!(resource, expected);
    }

    #[test]
    fn update_resource_by_key_value() {
        let doc_dir = "./.test/doc/update_resource_by_key_value/";
        let (path, key) = create_test_env_with_new_gpg_key(doc_dir.to_string());
        let doc = Document::new(
            DocumentNewOptions {
                directory: PathBuf::from(doc_dir),
                identity_fingerprint: key.fingerprint.clone(),
                name: String::from("name"),
            }).unwrap();

        let mut doc = doc.init(&key.fingerprint, &get_test_key().public_key).unwrap();

        let prop_key = key.fingerprint.clone() + ".x";
//        doc.update_resource_with_key_value("config", "39069565EA65A07AE1FBB4BB9B484B5D677BC2EA.fingerprint", "up").unwrap();
        doc.update_resource_with_key_value("config", prop_key.as_str(), "up").unwrap();
      //  doc.update_resource_with_key_value("config", "39069565EA65A07AE1FBB4BB9B484B5D677BC2EA.x.y", "up").unwrap();
    }

}
