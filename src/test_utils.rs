use std::fs;
use std::path::PathBuf;
use gpgme::ExportMode;
use crate::errors::Error;
use crate::gpg::{CreateUserArgs, Gpg, Key};


pub struct TestKey {
    pub fingerprint: String,
    pub public_key: String,
    pub secret_key: String,
}

pub fn get_test_key() -> TestKey {
    TestKey {
        fingerprint: "39069565EA65A07AE1FBB4BB9B484B5D677BC2EA".to_string(),
        public_key: r#"-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEYzxVCxYJKwYBBAHaRw8BAQdAIBFXz9lWTbRUZk8QdbtZIDzT8EksDBLUrD5I
o4wKjQi0GkFsaWNlIDxhbGljZUBjb2xvbWJhLmxpbms+iJAEExYIADgWIQQ5BpVl
6mWgeuH7tLubSEtdZ3vC6gUCYzxVCwIbAwULCQgHAgYVCgkICwIEFgIDAQIeAQIX
gAAKCRCbSEtdZ3vC6hwcAP9sPv78aC+t4MCasarWYv9FMtJ3aZMgpZchCCJD0b49
owEA9DSYX43Sf2btvmjjTRvmjSDdG/CzZ11/FZwCbRlJXws=
=JSAK
-----END PGP PUBLIC KEY BLOCK-----"#.to_string(),
        secret_key: r#"-----BEGIN PGP PRIVATE KEY BLOCK-----

lFgEYzxc1RYJKwYBBAHaRw8BAQdAHSpKLDT9Gjjl/Nl5VQGkhiq5MegUoBJpAQ5H
eQkevsQAAP4txwTfBeHqYsEvqCwAjTOVbCcH/fLG96FqJjby4YmA9RGetBpBbGlj
ZSA8YWxpY2VAY29sb21iYS5saW5rPoiQBBMWCAA4FiEEKx393uQpGJAg709HB6fY
x3X9CtQFAmM8XNUCGwMFCwkIBwIGFQoJCAsCBBYCAwECHgECF4AACgkQB6fYx3X9
CtThegD/b+aKV7KIZI6N3vLEoQay/sAgni0MJZkUR1ru4YiPK60A/3t1kGm+TIod
DdkLCFgUsP7kji+TiqIIcs0eg+UliL4P
=+mUj
-----END PGP PRIVATE KEY BLOCK-----"#.to_string(),
    }
}





pub fn create_test_env(test_data_path: String) -> PathBuf {
    let doc_dir = &PathBuf::from(test_data_path);
    fs::remove_dir_all(doc_dir).ok();
    fs::create_dir_all(doc_dir.as_path()).unwrap();
    std::env::set_var("GNUPGHOME", doc_dir.join(".keys").as_path());
    fs::create_dir_all(doc_dir.join(".keys").as_path()).unwrap();
    doc_dir.to_path_buf()
}

pub fn create_test_env_with_sample_gpg_key(test_data_path: String) -> (PathBuf, Key) {
    let doc_dir = create_test_env(test_data_path);
    let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
        .expect("Could create pgpme context from open pgp protocol");
    context.set_armor(true);
    let gpg_home = std::env::var("GNUPGHOME");
    context.set_engine_home_dir(gpg_home.unwrap());

    let result1 = context.import(get_test_key().public_key.as_bytes()).unwrap();
    let result = context.import(get_test_key().secret_key.as_bytes()).unwrap();

    let pub_key = context.get_key(get_test_key().fingerprint).unwrap();
    let key = Key {
        fingerprint: get_test_key().fingerprint,
        public: Some(pub_key),
    };

    let pub_key = context.get_secret_key(get_test_key().fingerprint).unwrap();

    (doc_dir, key)
}

pub fn create_armored_key() -> () {
    let (path, key) = create_test_env_with_sample_gpg_key("./.test/generate_keys/".to_string());
    let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
        .expect("Could create pgpme context from open pgp protocol");
    context.set_armor(true);
    let gpg_home = std::env::var("GNUPGHOME");
    if gpg_home.is_ok() {
        context.set_engine_home_dir(gpg_home.unwrap());
    }

    println!("fingerprint {}", key.fingerprint);

    let pub_key = context
        .get_key(key.fingerprint.clone())
        .map_err(|e| Error::GpgmeError(e)).unwrap();

    let mut data: Vec<u8> = Vec::new();
    context.export_keys(&[pub_key], gpgme::ExportMode::empty(), &mut data).expect("Could not export key");
    println!("{}", String::from_utf8(data).unwrap());

    context.set_key_list_mode(gpgme::KeyListMode::WITH_SECRET);
    let mut sec_data: Vec<u8> = Vec::new();
    context.export(Some(key.fingerprint.clone()), ExportMode::SECRET, &mut sec_data).unwrap();
    println!("{}", String::from_utf8(sec_data).unwrap());
}


/**

for some reason we can not read the secret key if we import from the armored content

called `Result::unwrap()` on an `Err` value: Error { source: Some("GPGME"), code: 16383, description: "End of file" }
thread 'document::tests::init_new_doc' panicked at 'called `Result::unwrap()` on an `Err` value: Error { source: Some("GPGME"), code: 16383, description: "End of file" }', src/test_utils.rs:69:70
stack backtrace:
*/

pub fn create_test_env_with_new_gpg_key(test_data_path: String) -> (PathBuf, Key) {
    let doc_dir = create_test_env(test_data_path);
    let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
        .expect("Could create pgpme context from open pgp protocol");
    context.set_armor(true);
    let gpg_home = std::env::var("GNUPGHOME");
    context.set_engine_home_dir(gpg_home.unwrap());

    let mut gpg = Gpg::new();
    let key = gpg.create_key(
        CreateUserArgs{ email: "alice@colomba.link", name: "Alice"}
    );
    let key = Gpg::key_with_public_key(&mut gpg, &key.as_ref().expect("a key")).unwrap();

    (doc_dir, key)
}

