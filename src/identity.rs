use crate::errors::Error;
use crate::gpg;
use crate::gpg::{CreateUserArgs, Gpg, Key};


pub struct Identity {}

const DEFAULT_GPG_HOME: &str = "./gpghome";

fn get_gpg_home(keyring_home_dir: Option<String>) -> String {
    match keyring_home_dir {
        Some(dir) => dir,
        None => String::from(DEFAULT_GPG_HOME)
    }
}

impl Identity {
    pub fn create_identity(keyring_home_dir: Option<String>) -> Result<Key, Error> {
        let home_dir = get_gpg_home(keyring_home_dir);

        let mut gpg = gpg::Gpg::new_with_custom_home(&home_dir);
        let key = gpg.create_key(
            CreateUserArgs{ email: "alice@colomba.link", name: "Alice"}
        );
        key
    }

    pub fn print_all_identities(keyring_home_dir: Option<String>) -> Result<(), Error> {
        let home_dir = get_gpg_home(keyring_home_dir);
        let mut gpg = gpg::Gpg::new_with_custom_home(&home_dir);
        let keys = gpg.get_all_public_keys().unwrap();

        match keys.len() {
            0 => println!("No keys found"),
            1 => println!("Found 1 key"),
            n => println!("Found {} keys", n)
        }

        println!();
        for key in keys {
            for (i, uid) in key.user_ids().enumerate() {
                println!("{}. Key" ,i+1);
                println!("----------------");
                println!("UID:\t\t {}", uid.id().unwrap());
                println!("Fingerprint:\t {}", key.fingerprint().unwrap());
            }
        }

        Ok(())
    }

}
