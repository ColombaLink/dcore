use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use git2::{BranchType, Repository, RepositoryInitOptions};
use yrs::updates::decoder::Decode;
use yrs::{Map, PrelimMap, Update};

use crate::document_utils::DocumentUtils;
use crate::errors::Error;
use crate::gpg::{Gpg, Key};
use crate::resource::Resource;
use crate::Identity;
use crate::sync_git::GitSync;

pub struct Document {
    pub name: String,
    pub repository: Repository,
    pub identity: Identity,
    pub gpg: Gpg,
    pub resources: HashMap<String, Resource>,
}



impl Document {
    pub fn add_resource(&mut self, p0: String) -> Result<(), Error> {
        if self.resources.contains_key(&p0) {
            return Err(Error::Other(
                "Document already initialized because the config resource exists".to_string(),
            ));
        }
        let mut resource = Resource::new(&p0);
        let update = resource.set_resource_meta(&p0).unwrap();

        let _ = &self.commit_update(&update, &resource);
        self.resources.insert(p0, resource);

        Ok(())
    }

    pub fn config_set_local_device(&self, device_name: &str) -> Result<(), Error> {
        // check that only allowed characters are used in device name (a-z, A-Z, 0-9, -)
        if !device_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(Error::Other(
                "Device name can only contain a-z, A-Z, 0-9, -".to_string(),
            ));
        }
        self.repository.config()?.set_str("user.device", device_name)?;
        Ok(())
    }

    pub fn config_get_local_device(&self) -> Result<String, Error> {
        let config =  self.repository.config().unwrap().snapshot().unwrap();

        match config.get_str("user.device") {
            Ok(device) => Ok(device.to_string()),
            Err(_) => Ok("device-0".to_string()),
        }
    }

    pub fn config_set_remote(&mut self, remote: &str) -> Result<(), Error> {
        let fingerprint = self.identity.get_fingerprint();
        let key = format!("{}.remote", fingerprint);
        self.update_resource_with_key_value("config", key.as_str(), remote).unwrap();
        Ok(())
    }

    pub fn clone(self, remote: &String) -> Result<(), Error> {
        self.repository.remote_set_url("origin", remote)?;
        GitSync::clone(self, remote);
        Ok(())
    }

    pub fn sync(self) -> Result<(), Error> {
        GitSync::sync( self)?;
        Ok(())
    }
}



unsafe impl Send for Document {}

pub struct DocumentInitOptionsIdentity {
    pub fingerprint: String,
}

pub struct DocumentInitOptions {
    pub directory: PathBuf,
    pub identity: DocumentInitOptionsIdentity,
}

pub struct DocumentNewOptions {
    pub directory: PathBuf,
    pub name: String,
    pub identity_fingerprint: String,
}

impl Document {
    pub fn new(options: DocumentNewOptions) -> Result<Document, Error> {
        let data_dir = PathBuf::from(options.directory).join("./.data");
        let repository = Repository::init_opts(&data_dir, &RepositoryInitOptions::new().bare(true))
            .map_err(|e| Error::GitError(e))?;
        let mut gpg = Gpg::new();
        let identity = Identity::from_fingerprint(&mut gpg, &options.identity_fingerprint).expect(
            ("Could not find the identity with the provided fingerprint ".to_string()
                + &options.identity_fingerprint)
                .as_str(),
        );

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
            return Err(Error::Other(
                "Document already initialized because the config resource exists".to_string(),
            ));
        }
        let mut resource = Resource::new(&String::from("config"));

        let update = resource
            .add_local_update(|mut transaction| {
                let config_root = transaction.get_map("root");

                let public_key = public_key.clone();
                let fingerprint = fingerprint.clone();
                config_root.insert(
                    &mut transaction,
                    fingerprint.as_str().to_owned(),
                    PrelimMap::<i32>::from(HashMap::default()),
                );
                let id_map = config_root
                    .get(fingerprint.as_str().to_owned().as_str())
                    .unwrap()
                    .to_ymap()
                    .unwrap();
                id_map.insert(
                    &mut transaction,
                    "fingerprint".to_string(),
                    fingerprint.as_str().to_owned(),
                );
                id_map.insert(
                    &mut transaction,
                    "public_key".to_string(),
                    public_key.as_str().to_owned(),
                );
                id_map.insert(
                    &mut transaction,
                    "alias".to_string(),
                    fingerprint.as_str().to_owned(),
                );

                transaction
            })
            .unwrap();

        let _ = &self.commit_update(&update, &resource);

        let mut resources = HashMap::new();
        resources.insert("config".to_string(), resource);

        Ok(Document {
            name: "name".to_string(),
            repository: self.repository,
            identity: self.identity,
            gpg: Gpg::new(),
            resources,
        })
    }

    fn commit_update(&self, update: &Vec<u8>, resource: &Resource) {
        DocumentUtils::commit_update(&self, resource, update.to_owned())
            .expect("TODO: panic message");
    }

    pub fn load(&mut self) -> Result<(), Error> {

        // load all resources from the repository
        // all the folders in the refs folder are resources

        let mut resources = HashSet::new();
        self
            .repository
            .references()
            .map_err(|e| Error::GitError(e))
            .unwrap()
            .for_each(|log| {
                let log_name = log.unwrap().name().unwrap().clone().to_string();
                // refs/local/{name}
                let resource_name = log_name.split("/").collect::<Vec<&str>>()[2].to_string();

                if resource_name == "main" || resource_name == "master" {
                    return;
                }
                resources.insert(resource_name);
            });


        for resource_name in resources {
            //println!("Loading resource: {}", resource_name);



            let mut resource_logs_head_oids = Vec::new();

            for log in self.repository.references().unwrap() {
                let log = log.unwrap();
                let log_name = log.name().unwrap();
                // refs/local/{name}
                let this_resource_name = log_name.split("/").collect::<Vec<&str>>()[2].to_string();
                if this_resource_name == resource_name {
                    resource_logs_head_oids.push(log.target().unwrap());
                }
            }

            let mut resource = Resource::new(&resource_name);
            let revwalk = &mut self
                .repository
                .revwalk()
                .map_err(|e| Error::GitError(e))
                .unwrap();
            revwalk
                .set_sorting(git2::Sort::REVERSE)
                .map_err(|e| Error::GitError(e))
                .unwrap();
            let mut t = resource.store.transact();
            for oid in resource_logs_head_oids {
                revwalk.push(oid).map_err(|e| Error::GitError(e)).unwrap();
                //println!("log: {}", oid.to_string());
                let updates = revwalk
                    .flat_map(|id| self.repository.find_commit(id.unwrap()))
                    .map(|commit| commit.tree().unwrap())
                    .map(|tree| {
                        tree.get_name("update")
                            .unwrap()
                            .to_object(&self.repository)
                            .unwrap()
                    })
                    .map(|object| object.peel_to_blob().unwrap())
                    .map(|blob| blob.content().to_vec())
                    .map(|content| Update::decode_v2(content.as_slice()).unwrap());

                let update = Update::merge_updates(updates);
                t.apply_update(update);
            }

            t.commit();
            self.resources.insert(resource_name, resource);

        }

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
    pub fn update_resource_with_key_value(
        &mut self,
        resource_name: &str,
        key: &str,
        value: &str,
    ) -> Result<(), Error> {
        let resource = self.resources.get_mut(resource_name).unwrap();
        let update = resource
            .add_local_update(|mut transaction| {
                let mut key_parts = key.split(".");
                let root_map = transaction.get_map("root");
                let mut current_map = root_map.clone();

                while let Some(key) = key_parts.next() {
                    if key_parts.clone().peekable().peek().is_some() {
                        // there will be a next key
                        // check if the current key already exists
                        //println!("key: {}", key);
                        match current_map.get(key) {
                            Some(map) => current_map = map.to_ymap().unwrap().clone(),
                            None => {
                                // this does not work correctly at the moment: it overwrites the the value...
                                // todo: fix nested key
                                let next_map = PrelimMap::<i32>::from(HashMap::default());
                                current_map.insert(&mut transaction, key.to_owned(), next_map);
                                current_map =
                                    current_map.get(key).unwrap().to_ymap().unwrap().clone();
                            }
                        }
                    } else {
                        // last key, so we reached the root
                        current_map.insert(&mut transaction, key.to_owned(), value.to_owned());
                    }
                }

                transaction
            })
            .unwrap();

        let resource1 = self.resources.get(resource_name).unwrap();
        self.commit_update(&update, &resource1);
        Ok(())
    }

    pub(crate) fn get_config(&self) -> Result<Map, Error>{
        let resource = self.resources.get("config").unwrap();
        Ok(resource.store.transact().get_map("root"))
    }

    pub(crate) fn config_get_remote(&self) -> Result<String, Error>{
        let config = self.get_config().unwrap();
        let fingerprint = self.identity.get_fingerprint();
        let user_config = config.get(fingerprint.as_str()).unwrap().to_ymap().unwrap();
        let remote = user_config.get("remote");
        match remote {
            Some(remote) => Ok(remote.to_string()),
            None => {
                let message = format!("Could not load remote.No remote configured for user {}", fingerprint);
                Err(Error::DcoreError(message))
            }
        }
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

    use crate::test_utils::{
        create_test_env, create_test_env_with_new_gpg_key, create_test_env_with_sample_gpg_key,
        create_test_env_with_test_gpg_key, get_test_key,
    };


    #[test]
    fn new_doc() {
        let doc_dir = "./.test/doc/new_doc/";
        create_test_env(&doc_dir.to_string());

        let _doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: get_test_key().fingerprint,
            name: String::from("name"),
        })
        .unwrap();
    }

    #[test]
    fn init_new_doc() {
        let doc_dir = "./.test/doc/init_new_doc/";
        let (_dir, key) = create_test_env_with_sample_gpg_key(doc_dir.to_string());

        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: key.fingerprint.clone(),
            name: String::from("test-doc1"),
        })
        .unwrap();

        let doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();

        let r = doc.resources.get("config").unwrap();
        let resource = r.store.transact().get_map("root").to_json();

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

        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: key.fingerprint.clone(),
            name: String::from("test-doc1"),
        })
        .unwrap();

        let doc = &mut doc
            .init(&key.fingerprint, &get_test_key().public_key)
            .unwrap();

        let doc_to_load = &mut Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: key.fingerprint.clone(),
            name: String::from("test-doc1"),
        })
        .unwrap();

        doc_to_load.load().unwrap();

        let r = doc_to_load.resources.get("config").unwrap();
        let resource = r.store.transact().get_map("root").to_json();

        let r = doc.resources.get("config").unwrap();
        let expected = r.store.transact().get_map("root").to_json();

        assert_eq!(resource, expected);
    }

    #[test]
    fn update_resource() {
        let doc_dir = "./.test/doc/init_new_doc/";
        create_test_env_with_sample_gpg_key(doc_dir.to_string());
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: get_test_key().fingerprint,
            name: String::from("name"),
        })
        .unwrap();

        let mut doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();

        let r = doc.resources.get_mut("config").unwrap();
        r.add_local_update(|mut transaction| {
            let config_root = transaction.get_map("root");

            config_root.insert(
                &mut transaction,
                "39069565EA65A07AE1FBB4BB9B484B5D677BC2EA",
                {
                    let mut user_conf = HashMap::new();
                    user_conf.insert("fingerprint".to_owned(), "up");
                    user_conf.insert("publicKey".to_owned(), "date");
                    user_conf
                },
            );

            transaction
        })
        .unwrap();

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

        let doc_to_load = &mut Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: get_test_key().fingerprint,
            name: String::from("test-doc1"),
        })
            .unwrap();

        doc_to_load.load().unwrap();

        let r = doc_to_load.resources.get("config").unwrap();
        let resource = r.store.transact().get_map("root").to_json();

        let r = doc.resources.get("config").unwrap();
        let expected = r.store.transact().get_map("root").to_json();

        assert_eq!(resource, expected);

    }

    #[test]
    fn add_resource() {
        let doc_dir = "./.test/doc/add_resource/";
        create_test_env_with_test_gpg_key(doc_dir.to_string());
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "A84E5D451E9E75B4791556896F45F34A926FBB70".to_string(),
            name: String::from("name"),
        })
            .unwrap();

        let mut doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();

        doc.resources.get_mut("config").unwrap();
        doc.add_resource("test".to_string()).unwrap();

        let result =  fs::read("./.test/doc/add_resource/.data/refs/local/test/A84E5D451E9E75B4791556896F45F34A926FBB70/device-0").unwrap();
        assert_eq!(result.len(), 41);
    }

    #[test]
    fn update_resource_by_key_value() {
        let doc_dir = "./.test/doc/update_resource_by_key_value/";
        let (path, key) = create_test_env_with_new_gpg_key(doc_dir.to_string());
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: key.fingerprint.clone(),
            name: String::from("name"),
        })
        .unwrap();

        let mut doc = doc
            .init(&key.fingerprint, &get_test_key().public_key)
            .unwrap();

        let prop_key = key.fingerprint.clone() + ".x";
        //        doc.update_resource_with_key_value("config", "39069565EA65A07AE1FBB4BB9B484B5D677BC2EA.fingerprint", "up").unwrap();
        doc.update_resource_with_key_value("config", prop_key.as_str(), "up")
            .unwrap();
        //  doc.update_resource_with_key_value("config", "39069565EA65A07AE1FBB4BB9B484B5D677BC2EA.x.y", "up").unwrap();
    }

    #[test]
    fn update_test_resource_with_key_value() {
        let doc_dir = "./.test/doc/update_test_resource_with_key_value/";
        create_test_env_with_test_gpg_key(doc_dir.to_string());
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "A84E5D451E9E75B4791556896F45F34A926FBB70".to_string(),
            name: String::from("name"),
        })
            .unwrap();

        let mut doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();

        doc.add_resource("test".to_string()).unwrap();

        let result =  fs::read("./.test/doc/update_test_resource_with_key_value/.data/refs/local/test/A84E5D451E9E75B4791556896F45F34A926FBB70/device-0").unwrap();
        assert_eq!(result.len(), 41);

        doc.update_resource_with_key_value("test", "entry", "1234").unwrap();

        let result = doc.resources.get("test").unwrap().get_content();
        assert_eq!(result, "{entry: 1234}");
    }

    #[test]
    fn reload_update_test_resource_with_key_value() {
        let doc_dir = "./.test/doc/reload_update_test_resource_with_key_value/";
        create_test_env_with_test_gpg_key(doc_dir.to_string());
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "A84E5D451E9E75B4791556896F45F34A926FBB70".to_string(),
            name: String::from("name"),
        })
            .unwrap();

        let mut doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();

        doc.add_resource("test".to_string()).unwrap();

        let result =  fs::read("./.test/doc/reload_update_test_resource_with_key_value/.data/refs/local/test/A84E5D451E9E75B4791556896F45F34A926FBB70/device-0").unwrap();
        assert_eq!(result.len(), 41);

        doc.update_resource_with_key_value("test", "test", "1234").unwrap();
        doc.update_resource_with_key_value("test", "nested.test", "1234").unwrap();

        let mut doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "A84E5D451E9E75B4791556896F45F34A926FBB70".to_string(),
            name: String::from("name"),
        })
            .unwrap();

        doc.load().unwrap();

        //let result = doc.resources.get("test").unwrap().get_content();
        //assert_eq!(result, "{test: 1234}");

        let r = doc.resources.get("test").unwrap();
        let b = r.get_content();
        assert_eq!(b, "{test: 1234, nested: {test: 1234}}");
    }


    #[test]
    fn config_set_device_name() {
        let doc_dir = "./.test/doc/config_set_device_name/";
        create_test_env_with_test_gpg_key(doc_dir.to_string());
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "A84E5D451E9E75B4791556896F45F34A926FBB70".to_string(),
            name: String::from("name"),
        }).unwrap();


        doc.config_set_local_device("dev0").unwrap();

        let mut doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();


        doc.add_resource("test".to_string()).unwrap();

        let result =  fs::read("./.test/doc/config_set_device_name/.data/refs/local/test/A84E5D451E9E75B4791556896F45F34A926FBB70/dev0").unwrap();
        assert_eq!(result.len(), 41);

    }

}
