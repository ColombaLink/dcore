use std::collections::HashMap;
use git2::{BlobWriter, Error, Repository, RepositoryInitOptions};
use std::path::{PathBuf};
use yrs::Transaction;
use crate::gpg::{Gpg, Key};
use crate::doc::{Doc};

pub struct Resource {
    pub name: String,
    pub store: yrs::Doc,
    local_transaction: Option<Transaction>,
    remote_transaction: Option<Transaction>
}



impl Resource {


    pub fn new(name: String) -> Resource {
        let store = yrs::Doc::new();
        Resource {
            name,
            store,
            local_transaction: None,
            remote_transaction: None
        }
    }

    pub fn from(store: yrs::Doc) -> Resource {
        let mut transaction = store.transact();
        let name = transaction.get_map("_resource_meta").get("name").unwrap().to_string();

        let mut remote_transaction = store.transact();
        Resource {
            name,
            store,
            local_transaction: Some(transaction),
            remote_transaction: Some(remote_transaction)
        }
    }



    pub fn set_resource_meta(&mut self, name: String) -> Result<Vec<u8>,Error> {
        self.update_resource(|t| {
            let resource_meta = t.get_map("_resource_meta");
            resource_meta
                .insert(t, "name".to_owned(),name.as_str() );
            t
        })
    }


    pub fn update_resource<F>(&mut self, update_func: F) -> Result<Vec<u8>, Error>
        where F: Fn(&mut Transaction) -> &Transaction {
        let mut transaction = match self.local_transaction.as_mut() {
            Some(t) => t,
            None => {
                self.local_transaction = Some(self.store.transact());
                self.local_transaction.as_mut().unwrap()
            }
        };
        update_func(&mut transaction);
        let update = transaction.encode_update_v2();
        transaction.commit();
        Ok((update))
    }

}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use crate::Doc;
    use crate::resource::Resource;

    #[test]
    fn init_new_doc() {
        let doc_dir = "./.test/doc/init_new_doc/";
        fs::remove_dir_all(doc_dir).ok();
        let doc = Doc::init(
            &crate::DocumentInitOptions{
                directory: PathBuf::from(doc_dir),
                identity: crate::doc::DocumentInitOptionsIdentity{
                    fingerprint: String::from("fingerprint"),
                    public_key: String::from("public_key"),
                }
            }
        ).unwrap();


        let mut resource = Resource::new(String::from("test"));

        resource.set_resource_meta("test".to_string()).unwrap();

        resource.update_resource(|resource| {
            println!("test");
            resource
        }).unwrap();

    }


    #[test]
    fn resource_from_store() {
        let doc_dir = "./.test/doc/resource_from_store/";
        fs::remove_dir_all(doc_dir).ok();
        let doc = Doc::init(
            &crate::DocumentInitOptions{
                directory: PathBuf::from(doc_dir),
                identity: crate::doc::DocumentInitOptionsIdentity{
                    fingerprint: String::from("fingerprint"),
                    public_key: String::from("public_key"),
                }
            }
        ).unwrap();


        let mut resource = Resource::new(String::from("test"));

        resource.set_resource_meta("test".to_string()).unwrap();


        let reloaded_resource = Resource::from(resource.store);
        assert_eq!(reloaded_resource.name, "test");

    }


}
