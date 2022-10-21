use crate::Document;
use crate::errors::Error;

struct GitSync;

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

        Ok(())
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
    fn sync_with_git() {
        let doc_dir = "./.test/sync_git/sync_with_git/";
        create_test_env_with_test_gpg_key(doc_dir.to_string());
        let doc = Document::new(DocumentNewOptions {
            directory: PathBuf::from(doc_dir),
            identity_fingerprint: "A84E5D451E9E75B4791556896F45F34A926FBB70".to_string(),
            name: String::from("name"),
        }).unwrap();
        let mut doc = doc
            .init(&get_test_key().fingerprint, &get_test_key().public_key)
            .unwrap();

        doc.config_set_remote("https://example.remote").unwrap();

        //let content = doc.resources.get("config").unwrap().get_content();
        //assert_eq!(content, "..., just for debugging");

        let remote = doc.config_get_remote().unwrap();
        assert_eq!(remote, "https://example.remote");
    }

}
