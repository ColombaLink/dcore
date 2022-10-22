use std::borrow::BorrowMut;

use std::fmt::Write;
use std::str::from_utf8;
use std::time::Duration;

use gpgme::{CreateKeyFlags, ExportMode};

use crate::errors::Error;

use crate::Identity;

pub struct Gpg {
    context: gpgme::Context,
}

impl Gpg {

    // todo: we need the key in the armored ssh format
    pub(crate) fn get_armored_public_key(fingerprint: &str) -> Result<String, Error> {
        let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
            .expect("Could create pgpme context from open pgp protocol");
        context.set_armor(true);
        let key = context
            .get_key(fingerprint)
            .unwrap();
        let mut data: Vec<u8> = Vec::new();
        context
            .export_keys(&[key], gpgme::ExportMode::empty(), &mut data)
            .expect("Could not export key");
        Ok(String::from_utf8(data).unwrap())
    }

    // todo: maybe pass the reference to the str instead of returning a String (=avoid heap)?
    pub(crate) fn get_armored_private_key(fingerprint: &str) -> Result<String, Error> {
        let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
            .expect("Could create pgpme context from open pgp protocol");
        context.set_armor(true);
        let key = context
            .get_key(fingerprint)
            .unwrap();
        context
            .set_key_list_mode(gpgme::KeyListMode::WITH_SECRET)
            .unwrap();
        let mut data: Vec<u8> = Vec::new();
        context
            .export(
                Some(fingerprint),
                ExportMode::SECRET,
                &mut data,
            )
            .unwrap();
        Ok(String::from_utf8(data).unwrap())
    }
}

impl Gpg {
    #[allow(dead_code)]
    pub fn encypt(
        &mut self,
        update: &Vec<u8>,
        identity: &Identity,
    ) -> Result<String, Error> {
        let signing_key = identity.get_fingerprint();
        let ctx = self.context.borrow_mut();

        let key = ctx.borrow_mut().get_key(signing_key)?;
        let mut ciphertext = Vec::new();
        ctx.encrypt_with_flags(
            Some(&key),
            update,
            &mut ciphertext,
            gpgme::EncryptFlags::ALWAYS_TRUST,
        )
        .unwrap();
        Ok(String::from_utf8(ciphertext).unwrap())
    }


 pub fn decrypt(&mut self, ciphertext: &Vec<u8>) -> Result<String, Error> {
    let ctx = self.context.borrow_mut();
        let mut plaintext = Vec::new();
    ctx.decrypt_and_verify_with_flags(
        ciphertext,
        &mut plaintext,
        gpgme::DecryptFlags::VERIFY,
    )
        .unwrap();
    Ok(String::from_utf8(plaintext.clone()).unwrap())
}
}

pub struct Key {
    /// A de-armored public key
    pub public: Option<gpgme::Key>,
    /// The finger print of the key
    pub fingerprint: String,
}

pub struct CreateUserArgs<'a> {
    pub name: &'a str,
    pub email: &'a str,
}

/*
pub struct Options {
    pub public: Option<String>,
    pub private: Option<String>,
    pub key: KeyType
}

impl Default for Options {
    fn default() -> Self {
        let client_id: u32 = 1;
    }
}
*/

impl Gpg {
    pub fn new() -> Self {
        let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
            .expect("Could create pgpme context from open pgp protocol");
        context.set_armor(true);

        let gpg_home = std::env::var("GNUPGHOME");
        if gpg_home.is_ok() {
            context.set_engine_home_dir(gpg_home.unwrap()).unwrap();
        }

        Gpg { context }
    }

    pub fn new_with_custom_home(home: &str) -> Self {
        println!("Creating new gpg instance with home dir: {}", home);
        let mut gpg = Gpg::new();
        gpg.context
            .set_engine_home_dir(home)
            .expect("Could not set gpg engine home dir");
        gpg
    }

    fn create_new_ed25519_key(&mut self, user: CreateUserArgs) -> Result<Key, Error> {
        let mut user_id = String::new();
        write!(user_id, "{} <{}>", user.name, user.email).unwrap();
        let key_gen_result = match self.context.create_key_with_flags(
            user_id,
            "default",
           // "ed25519",
            Duration::from_secs(0), // did not figure out to import default
            gpgme::CreateKeyFlags::from(CreateKeyFlags::NOEXPIRE),
        ) {
            Ok(r) => Result::Ok(r),
            Err(e) => Result::Err(Error::GpgmeError(e)),
        }?;

        let fingerprint = match key_gen_result.fingerprint() {
            Ok(r) => Result::Ok(String::from(r)),
            // todo: handle this error properly, don't know how to handle this error..
            Err(e) => Result::Err(Error::Utf8Error(e.expect(""))),
        }?;
        Ok(Key {
            fingerprint,
            public: None,
        })
    }

    pub fn create_key(&mut self, user: CreateUserArgs) -> Result<Key, Error> {
        self.create_new_ed25519_key(user)
    }

    pub fn get_all_public_keys(&mut self) -> Result<Vec<gpgme::Key>, Error> {
        Ok(self
            .context
            .keys()
            .map_err(|e| Error::GpgmeError(e))?
            .into_iter()
            .filter_map(|key| key.ok())
            .map(|k| k.into())
            .collect())
    }

    pub fn key_with_public_key(&mut self, key: &Key) -> Result<Key, Error> {
        let public_key = self
            .context
            .get_key(key.fingerprint.as_str())
            .map_err(|e| Error::GpgmeError(e))?;
        Ok(Key {
            public: Some(public_key),
            fingerprint: key.fingerprint.clone(),
        })
    }

    pub fn get_public_key(&mut self, fingerprint: &str) -> Result<Key, Error> {
        let public_key = self
            .context
            .get_key(fingerprint)
            .map_err(|e| Error::GpgmeError(e))?;
        Ok(Key {
            public: Some(public_key),
            fingerprint: String::from(fingerprint),
        })
    }

    pub fn sign_string(&mut self, commit: &String, identity: &Identity) -> Result<String, Error> {
        let signing_key = identity.get_fingerprint();
        let ctx = self.context.borrow_mut();

        let key = ctx.borrow_mut().get_secret_key(signing_key)?;
        ctx.add_signer(&key)?;
        let mut output = Vec::new();
        let signature = ctx.sign_detached(commit.clone(), &mut output);

        if signature.is_err() {
            return Err(Error::GpgmeError(signature.unwrap_err()));
        }

        Ok(String::from(std::str::from_utf8(&output).unwrap()))
    }

    pub fn get_public_key_by_identity(&mut self, identity: &Identity) -> Result<Vec<u8>, Error> {
        let fingerprint = identity.get_fingerprint();
        // Find the GPGME key to export
        let key = self
            .context
            .get_key(fingerprint)
            .map_err(|e| Error::GpgmeError(e))
            .unwrap();

        let mut data: Vec<u8> = Vec::new();
        let cached_armor = self.context.armor();
        self.context.set_armor(true);
        self.context
            .export_keys(&[key], gpgme::ExportMode::empty(), &mut data)?;
        self.context.set_armor(cached_armor);

        let data_str = from_utf8(&data).expect("exported key is invalid UTF-8");
        assert!(
            !data_str.contains("PRIVATE KEY"),
            "The exported key contains a private key, blocked to prevent leaking secret key"
        );
        assert!(
            data_str.contains("PUBLIC KEY"),
            "The exported key must contain PUBLIC KEY. Something is wrong gpgme exported public key."
        );
        Ok(data)
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use gpgme::CreateKeyFlags;

    use crate::errors::Error;
    use crate::gpg::{CreateUserArgs, Gpg};
    use crate::test_utils::{create_armored_key, create_test_env, create_test_env_with_sample_gpg_key, key, rsa_key};
    use crate::Identity;

    #[test]
    fn create_armored_keys_for_tests() {
        create_armored_key()
    }

    #[test]
    fn create_new_gpg() {
        let gpg = Gpg::new();
        assert_eq!(gpg.context.armor(), true);
    }

    #[test]
    fn add_new_gpg_key() {
        let mut gpg = Gpg::new();
        let key = gpg.create_key(CreateUserArgs {
            email: "alice@colomba.link",
            name: "Alice",
        });
        assert_eq!(key.expect("a key").fingerprint.len(), 40);
    }

    #[test]
    fn get_email() {
        create_test_env("./.test/gpg/get_email");
        let mut gpg = Gpg::new();
        let key = gpg.create_key(CreateUserArgs {
            email: "alice@colomba.link",
            name: "Alice",
        });
        let k = gpg
            .get_public_key(key.unwrap().fingerprint.as_str())
            .unwrap();
        let id = k
            .public
            .expect("Key should have public key")
            .user_ids()
            .next()
            .unwrap()
            .email()
            .expect("Key should have email")
            .to_string()
            .to_owned();
        assert_eq!(id, "");
        //  assert_eq!(key.expect("a key").fingerprint.len(), 40);
    }

    // https://github.com/orhun/gpg-tui/blob/580d436bf296a0c4c70193f5b31b1334fd771968/src/app/launcher.rs

    #[test]
    fn with_public_key() {
        create_test_env("./.test/gpg/with_public_key");
        let mut gpg = Gpg::new();
        let key = gpg.create_key(CreateUserArgs {
            email: "alice@colomba.link",
            name: "Alice",
        });
        let key_with_public_key = Gpg::key_with_public_key(&mut gpg, &key.as_ref().expect("a key"));

        // todo: understand - why do we need as_ref() here? can this be avoided?
        assert_eq!(key_with_public_key.as_ref().unwrap().public.is_some(), true);
        assert_eq!(
            key_with_public_key.as_ref().unwrap().fingerprint,
            key.expect("a key").fingerprint
        );
    }
    /*
        let key = gpg.create_key(
            CreateUserArgs{ email: "alice@colomba.link", name: "Alice 00"}
        ).expect("key must be defined");
        let x = gpg.context.get_key(key.fingerprint);
    */

    #[test]
    fn as_libp2p_keypair() {
        // use libp2p::identity::ed25519::Keypair;

        // 1. generate a key pair with gpg (ed25519)

        let mut gpg = Gpg::new();
        let _key = gpg.create_key(CreateUserArgs {
            email: "alice@colomba.link",
            name: "Alice",
        });

        // 2. get the private key components from gpg such that we can map it to libp2p::identity::ed25519::Keypair
        // gpg.context.get_key("fingerprint").unwrap().primary_key().unwrap().as_raw().curve.key()

        // 3. create a libp2p::identity::ed25519::Keypair from the private key components

        // 4. sing some data with the gpg key pair and verify it with the libp2p::identity::ed25519::Keypair
    }

    #[test]
    fn test_delete_keys() {
        let gpghome = "./.test/gpg/delete_keys/gpghome";
        std::fs::remove_dir_all(gpghome);
        std::fs::create_dir_all(gpghome);

        let mut gpg = Gpg::new_with_custom_home(gpghome);
        //let keys = Gpg::get_all_public_keys(gpg.context.borrow_mut()).unwrap();
        let keys = gpg.get_all_public_keys().unwrap();
        for key in keys {
            println!("key: {}", key.fingerprint().unwrap());
            gpg.context.delete_key(&key).unwrap();
        }

        // todo: fix delete keys
        /*
        called `Result::unwrap()` on an `Err` value: Error { source: Some("GPGME"), code: 70, description: "Conflicting use" }
            thread 'gpg::tests::test_delete_keys' panicked at 'called `Result::unwrap()` on an `Err` value: Error { source: Some("GPGME"), code: 70, description: "Conflicting use" }', src/gpg.rs:138:42
            stack backtrace:
         */
    }

    #[test]
    fn test_gpg_sign() {
        create_test_env("./.test/gpg/sign/gpghome");
        let mut gpg = Gpg::new();
        let key = gpg.create_key(CreateUserArgs {
            email: "alice@colomba.link",
            name: "Alice",
        });
        let key_with_public_key = Gpg::key_with_public_key(&mut gpg, &key.as_ref().expect("a key"));

        let identity = Identity::from_key(key_with_public_key.unwrap());
        let content = String::from("hello world");
        let signature = gpg.sign_string(&content, &identity);
        assert_eq!(signature.is_ok(), true);
        assert_eq!(signature.unwrap().len(), 228);
    }

    #[test]
    fn test_get_armored_public_key() {
        let (_path, key) =
            create_test_env_with_sample_gpg_key("./.test/gpg/get_armored_public_key/".to_string());

        let mut gpg = Gpg::new();
        let identity = &Identity::from_key(key);
        let armored_public_key = gpg.get_public_key_by_identity(identity).unwrap();
        assert_eq!(armored_public_key.len(), 388);
    }

    #[test]
    fn test_encryption() {
        create_test_env("./.test/gpg/sign/gpghome");
        let mut gpg = Gpg::new();

        let user_id = String::from("Alice <alice@colomba.link>");
        let key_gen_result = match gpg.context.create_key_with_flags(
            user_id,
            "default",
            Duration::from_secs(0), // did not figure out to import default
            gpgme::CreateKeyFlags::from(CreateKeyFlags::NOEXPIRE),
        ) {
            Ok(r) => Result::Ok(r),
            Err(e) => Result::Err(Error::GpgmeError(e)),
        }
        .expect("key must be created");

        let fingerprint = match key_gen_result.fingerprint() {
            Ok(r) => Result::Ok(String::from(r)),
            // todo: handle this error properly, don't know how to handle this error..
            Err(e) => Result::Err(Error::Utf8Error(e.expect(""))),
        }
        .expect("fingerprint must be defined");

        let key_with_public_key = gpg.get_public_key(&fingerprint).unwrap();
        let identity = Identity::from_key(key_with_public_key);
        let content = String::from("hello world").as_bytes().to_vec();
        let r = gpg
            .encypt(&content, &identity)
            .expect("encryption must work");
        assert_eq!(r.len(), 699);
    }


    #[test]
    fn test_encryption_and_decryption() {
        create_test_env("./.test/gpg/sign/gpghome");
        let mut gpg = Gpg::new();

        let user_id = String::from("Alice <alice@colomba.link>");
        let key_gen_result = match gpg.context.create_key_with_flags(
            user_id,
            "default",
            Duration::from_secs(0), // did not figure out to import default
            gpgme::CreateKeyFlags::from(CreateKeyFlags::NOEXPIRE),
        ) {
            Ok(r) => Result::Ok(r),
            Err(e) => Result::Err(Error::GpgmeError(e)),
        }
            .expect("key must be created");

        let fingerprint = match key_gen_result.fingerprint() {
            Ok(r) => Result::Ok(String::from(r)),
            // todo: handle this error properly, don't know how to handle this error..
            Err(e) => Result::Err(Error::Utf8Error(e.expect(""))),
        }
            .expect("fingerprint must be defined");

        let key_with_public_key = gpg.get_public_key(&fingerprint).unwrap();
        let identity = Identity::from_key(key_with_public_key);
        let content = String::from("hello world").as_bytes().to_vec();
        let r = gpg
            .encypt(&content, &identity)
            .expect("encryption must work");

        let decrypted = gpg.decrypt(&r.as_bytes().to_vec()       ).expect("decryption must work");

        assert_eq!(decrypted, "hello world");
        assert_eq!(r.len(), 699);
    }


    #[test]
    fn test_go() {
        //key()
        rsa_key()
    }
}
