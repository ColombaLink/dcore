use std::collections::HashMap;
use git2::{BlobWriter, Error, Repository, RepositoryInitOptions};
use std::path::{PathBuf};
use crate::gpg::{Gpg, Key};
use crate::resource::Resource;

pub struct Doc {
    repository: Repository,
    identity: Key,
    gpg: Gpg,
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



  //      resource.add_resource();

    //    let id = repo.blob(&config).unwrap();
   //     println!("{}", id);

        Ok(Doc {
            repository: repo,
            identity: Key {
                public: None, // todo: set pk
                fingerprint: args.identity.fingerprint.clone(),
            },
            gpg: Gpg::new(),
        })
    }

}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use crate::Doc;

    #[test]
    fn init_new_doc() {
        let doc_dir = "./.test/doc/init_new_doc/";
        fs::remove_dir_all(doc_dir).ok();
        Doc::init(
            &crate::DocumentInitOptions{
                directory: PathBuf::from(doc_dir),
                identity: crate::doc::DocumentInitOptionsIdentity{
                    fingerprint: String::from("fingerprint"),
                    public_key: String::from("public_key"),
                }
            }).unwrap();

    }
}
