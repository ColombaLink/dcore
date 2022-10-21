use std::collections::HashMap;

use git2::Error;
use yrs::{Transaction, UpdateEvent};

use crate::event::{EventHandler, Subscription};

pub struct Resource {
    pub name: String,
    pub store: yrs::Doc,
    local_transaction: Option<EventHandler<UpdateEvent>>,
    pub local_transaction_subscriptions: HashMap<u32, Subscription<UpdateEvent>>,
}

impl Resource {
    pub fn new(name: &String) -> Resource {
        let store = yrs::Doc::new();
        Resource {
            name: name.clone(),
            store,
            local_transaction: None,
            local_transaction_subscriptions: HashMap::new(),
        }
    }

    pub fn from(store: yrs::Doc) -> Resource {
        let mut transaction = store.transact();
        let name = transaction
            .get_map("_resource_meta")
            .get("name")
            .unwrap()
            .to_string();

        let _remote_transaction = store.transact();
        Resource {
            name,
            store,
            local_transaction: None,
            local_transaction_subscriptions: HashMap::new(),
        }
    }

    pub fn set_resource_meta(&mut self, name: &String) -> Result<Vec<u8>, Error> {
        self.add_local_update(|t| {
            let resource_meta = t.get_map("_resource_meta");
            resource_meta.insert(t, "name".to_owned(), name.as_str());
            t
        })
    }

    pub fn add_local_update<F>(&mut self, update_func: F) -> Result<Vec<u8>, Error>
    where
        F: Fn(&mut Transaction) -> &Transaction,
    {
        let mut transaction = self.store.transact();
        update_func(&mut transaction);
        let update = transaction.encode_update_v2();
        transaction.commit();
        let eh = self.local_transaction.get_or_insert_with(EventHandler::new);
        eh.publish(
            &transaction,
            &UpdateEvent {
                update: update.clone(),
            },
        );
        Ok(update)
    }

    pub fn observe_local_transactions<F>(&mut self, f: F) -> Subscription<UpdateEvent>
    where
        F: Fn(&Transaction, &UpdateEvent) -> () + 'static,
    {
        let eh = self.local_transaction.get_or_insert_with(EventHandler::new);
        eh.subscribe(f)
    }

    pub fn get_content(&self) -> String {
        let mut transaction = self.store.transact();
        let content = transaction.get_map("root").to_json().to_string();
        content
    }
}

#[cfg(test)]
mod tests {

    use std::fs;
    use std::path::PathBuf;

    use crate::document::DocumentNewOptions;
    use crate::resource::Resource;
    use crate::test_utils::get_test_key;
    use crate::Document;

    #[test]
    fn init_new() {
        let doc_dir = "./.test/doc/init_new_doc/";
        fs::remove_dir_all(doc_dir).ok();
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "todo".to_string(),
            name: String::from("name"),
        })
        .unwrap();
        let _doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();

        let mut resource = Resource::new(&String::from("config"));
        resource.set_resource_meta(&"config".to_string()).unwrap();

        resource
            .add_local_update(|resource| {
                println!("test");
                resource
            })
            .unwrap();
    }

    #[test]
    fn resource_from_store() {
        let doc_dir = "./.test/doc/resource_from_store/";
        fs::remove_dir_all(doc_dir).ok();
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "todo".to_string(),
            name: String::from("name"),
        })
        .unwrap();
        let _doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();

        let mut resource = Resource::new(&String::from("test"));

        resource.set_resource_meta(&"test".to_string()).unwrap();

        let reloaded_resource = Resource::from(resource.store);
        assert_eq!(reloaded_resource.name, "test");
    }

    #[test]
    fn subscribe_to_local_transaction_context() {
        let doc_dir = "./.test/doc/resource_from_store/";
        fs::remove_dir_all(doc_dir).ok();
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "todo".to_string(),
            name: String::from("name"),
        })
        .unwrap();
        let _doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();

        let mut resource = Resource::new(&String::from("test"));

        let _sub = resource.observe_local_transactions(|_trans, _u| {
            println!("update");
        });

        resource.set_resource_meta(&"test".to_string()).unwrap();

        let _eh = resource
            .local_transaction
            .expect("can not get eventhandler");

        // todo: find a way to test this
        // the function is private ...
        // assert_eq!(eh.subscription_count(), 0);
    }
}
