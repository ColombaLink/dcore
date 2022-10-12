use crate::errors::{Error};
use crate::gpg;
use crate::gpg::{CreateUserArgs, Gpg, Key};


pub struct Identity {
    key: Key,
}

impl Identity {
    pub(crate) fn get_key(self) -> Key {
        self.key
    }
}

impl Identity {
    pub(crate) fn from_fingerprint(gpg: &mut Gpg, fingerprint: &String) -> Result<Identity, Error> {
        let key = gpg.get_public_key(fingerprint)?;
        Ok(Identity {
            key
        })
    }
}

impl Identity {
    pub(crate) fn from_key(key: Key) -> Identity {
        Identity { key }
    }
}


impl Identity {
    pub fn create_identity(keyring_home_dir: Option<String>, user_name: &str, user_email: &str) -> Result<Identity, Error> {
        let mut gpg = gpg::Gpg::new();
        let key = gpg.create_key(
            CreateUserArgs{ email: user_name, name: user_email}
        ).expect("Could not create the key with the provided options.");
        Ok(Identity { key: key })
    }

    pub fn print_all_identities(keyring_home_dir: Option<String>) -> Result<(), Error> {
        let mut gpg = gpg::Gpg::new();
        let keys = gpg.get_all_public_keys().unwrap();

        match keys.len() {
            0 => println!("No keys found"),
            1 => println!("Found 1 key"),
            n => println!("Found {} keys", n)
        }

        println!();
        for (i ,key) in keys.iter().enumerate() {
            println!("{}. Key" ,i+1);
            for uid in key.user_ids() {
                println!("----------------");
                println!("UID:\t\t {}", uid.id().unwrap());
                println!("Fingerprint:\t {}", key.fingerprint().unwrap());
            }
        }

        Ok(())
    }

    pub fn get_fingerprint(&self) -> String {
        self.key.fingerprint.clone()
    }
}


pub struct GetIdentityArgs {
    pub keyring_home_dir: Option<String>,
    pub fingerprint: String,
}

impl Identity {
    pub fn get_identity(args: GetIdentityArgs) -> Result<Key, Error> {
        let mut gpg = gpg::Gpg::new();
        let key = gpg.get_public_key(&args.fingerprint);
        key
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::{Borrow, BorrowMut};
    use std::time::Duration;
    use gpgme::{Context, CreateKeyFlags, Key};
    use gpgme::context::Keys;
    use gpgme::PinentryMode::Default;
    use crate::gpg::{CreateUserArgs, Gpg};
    use crate::Identity;
    use crate::identity::GetIdentityArgs;

    #[test]
    fn get_identity() {
        let gpghome = "./.test/identity/get_identity/gpghome";
        std::fs::remove_dir_all(gpghome);
        std::fs::create_dir_all(gpghome);

        let mut gpg = Gpg::new_with_custom_home("./.test/identity/get_identity/gpghome");
        let key = gpg.create_key(
            CreateUserArgs{ email: "alice@colomba.link", name: "Alice"}
        ).expect("create key");

        let identity = Identity::get_identity(GetIdentityArgs{
            keyring_home_dir: Some(String::from(gpghome)),
            fingerprint: key.fingerprint.clone()
        }).expect("get identity");

        assert_eq!(key.fingerprint, identity.fingerprint);
    }
}
