use crate::errors::Error;
use crate::gpg;
use crate::gpg::{CreateUserArgs, Gpg, Key};


pub struct Identity {}

impl Identity {
    pub fn create_identity(keyring_home_dir: Option<String>) -> Result<Key, Error> {
        let home_dir = match keyring_home_dir {
            Some(dir) => dir,
            None => String::from("./gpghome")
        };

        let mut gpg = gpg::Gpg::new_with_custom_home(&home_dir);
        let key = gpg.create_key(
            CreateUserArgs{ email: "alice@colomba.link", name: "Alice"}
        );
        key
    }
}
