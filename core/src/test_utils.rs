use fs_extra::dir::CopyOptions;
use std::fs;
use std::path::PathBuf;

use gpgme::ExportMode;
use sequoia_openpgp::parse::Parse;
use sequoia_openpgp::serialize::MarshalInto;
use sequoia_openpgp::Cert;

use crate::errors::Error;
use crate::gpg::{CreateUserArgs, Gpg, Key};

#[allow(dead_code)]
pub struct TestKey {
    pub fingerprint: String,
    pub public_key: String,
    pub secret_key: String,
}

#[allow(dead_code)]
pub fn get_test_key() -> TestKey {
    TestKey {
        fingerprint: "A84E5D451E9E75B4791556896F45F34A926FBB70".to_string(),
        public_key: r#"-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEY1FU4BYJKwYBBAHaRw8BAQdAKSqrB/NgijxUNMK0HHdrBGKBc812PFgBMm50
nIPKIA60GUFsaWNlIDxpbmZvQGNvbG9tYmEubGluaz6IkAQTFggAOBYhBKhOXUUe
nnW0eRVWiW9F80qSb7twBQJjUVTgAhsDBQsJCAcCBhUKCQgLAgQWAgMBAh4BAheA
AAoJEG9F80qSb7twJNQBAJ0WiFnWCG+1Cbk1etUnEreg49KzEnmIYebcZyV6yEXj
AP9TdoNQJnwIMQFYaSohawMwecp/A+F51Y0Pn90pX0vtDw==
=GYnp
-----END PGP PUBLIC KEY BLOCK-----"#
            .to_string(),
        secret_key: r#"-----BEGIN PGP PRIVATE KEY BLOCK-----

lFgEY1FU4BYJKwYBBAHaRw8BAQdAKSqrB/NgijxUNMK0HHdrBGKBc812PFgBMm50
nIPKIA4AAP9HB/Vo+ozhO0ehgZS8KzKmJx8cLxkebp41XFn0iQPJaQ54tBlBbGlj
ZSA8aW5mb0Bjb2xvbWJhLmxpbms+iJAEExYIADgWIQSoTl1FHp51tHkVVolvRfNK
km+7cAUCY1FU4AIbAwULCQgHAgYVCgkICwIEFgIDAQIeAQIXgAAKCRBvRfNKkm+7
cCTUAQCdFohZ1ghvtQm5NXrVJxK3oOPSsxJ5iGHm3GcleshF4wD/U3aDUCZ8CDEB
WGkqIWsDMHnKfwPhedWND5/dKV9L7Q8=
=Q3aJ
-----END PGP PRIVATE KEY BLOCK-----
"#
            .to_string(),
    }
}

#[allow(dead_code)]
pub fn create_test_env(test_data_path: &str) -> PathBuf {
    let doc_dir = &PathBuf::from(test_data_path);
    fs::remove_dir_all(doc_dir).ok();
    fs::create_dir_all(doc_dir.as_path()).unwrap();
    let key_dir = doc_dir.join(".key");
    let key_dir_str = key_dir.to_str().unwrap();
    std::env::set_var("GNUPGHOME", key_dir_str);
    fs::create_dir_all(key_dir).unwrap();
    doc_dir.join(".keys").as_path().to_path_buf()
}

#[allow(dead_code)]
pub fn create_test_env_with_sample_gpg_key(test_data_path: String) -> (PathBuf, Key) {
    let doc_dir = create_test_env(&test_data_path);
    let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
        .expect("Could create pgpme context from open pgp protocol");
    context.set_armor(true);
    let gpg_home = std::env::var("GNUPGHOME");
    context.set_engine_home_dir(gpg_home.unwrap()).unwrap();
    context
        .set_key_list_mode(gpgme::KeyListMode::WITH_SECRET)
        .unwrap();
    let _result1 = context
        .import(get_test_key().public_key.as_bytes())
        .unwrap();
    let _result = context
        .import(get_test_key().secret_key.as_bytes())
        .unwrap();

    let pub_key = context.get_key(get_test_key().fingerprint).unwrap();
    let key = Key {
        fingerprint: get_test_key().fingerprint,
        public: Some(pub_key),
    };

    (doc_dir, key)
}

#[allow(dead_code)]
pub fn create_armored_key() -> () {
    let (_path, key) = create_test_env_with_sample_gpg_key("./.test/generate_keys/".to_string());
    let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
        .expect("Could create pgpme context from open pgp protocol");
    context.set_armor(true);
    let gpg_home = std::env::var("GNUPGHOME");
    if gpg_home.is_ok() {
        context.set_engine_home_dir(gpg_home.unwrap()).unwrap();
    }

    println!("fingerprint {}", key.fingerprint);

    let pub_key = context
        .get_key(key.fingerprint.clone())
        .map_err(|e| Error::GpgmeError(e))
        .unwrap();

    let mut data: Vec<u8> = Vec::new();
    context
        .export_keys(&[pub_key], gpgme::ExportMode::empty(), &mut data)
        .expect("Could not export key");
    println!("{}", String::from_utf8(data).unwrap());

    context
        .set_key_list_mode(gpgme::KeyListMode::WITH_SECRET)
        .unwrap();
    let mut sec_data: Vec<u8> = Vec::new();
    context
        .export(
            Some(key.fingerprint.clone()),
            ExportMode::SECRET,
            &mut sec_data,
        )
        .unwrap();
    println!("{}", String::from_utf8(sec_data).unwrap());
}

#[allow(dead_code)]
pub fn key() -> () {
    let (_path, key) = create_test_env_with_sample_gpg_key("./.test/generate_keys/".to_string());
    let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
        .expect("Could create pgpme context from open pgp protocol");
    context.set_armor(true);
    let gpg_home = std::env::var("GNUPGHOME");
    if gpg_home.is_ok() {
        context.set_engine_home_dir(gpg_home.unwrap()).unwrap();
    }

    println!("fingerprint {}", key.fingerprint);

    let pub_key = context
        .get_key(key.fingerprint.clone())
        .map_err(|e| Error::GpgmeError(e))
        .unwrap();

    let mut data: Vec<u8> = Vec::new();
    context
        .export_keys(&[pub_key], gpgme::ExportMode::MINIMAL, &mut data)
        .expect("Could not export key");

    let y = &mut data.clone();
    let mut r = Cert::from_bytes(y);

    // let x = r.unwrap().fingerprint();
    let x = r
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .mpis()
        .clone()
        .to_vec()
        .unwrap();
    println!("fingerprint {}", x.len());
    // println!("{}", String::from_utf8(data).unwrap());
}

/**

for some reason we can not read the secret key if we import from the armored content

called `Result::unwrap()` on an `Err` value: Error { source: Some("GPGME"), code: 16383, description: "End of file" }
thread 'document::tests::init_new_doc' panicked at 'called `Result::unwrap()` on an `Err` value: Error { source: Some("GPGME"), code: 16383, description: "End of file" }', src/test_utils.rs:69:70
stack backtrace:
*/

#[allow(dead_code)]
pub fn create_test_env_with_new_gpg_key(test_data_path: String) -> (PathBuf, Key) {
    let doc_dir = create_test_env(&test_data_path);
    let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
        .expect("Could create pgpme context from open pgp protocol");
    context.set_armor(true);
    let gpg_home = std::env::var("GNUPGHOME");
    context.set_engine_home_dir(gpg_home.unwrap()).unwrap();

    let mut gpg = Gpg::new();
    let key = gpg.create_key(CreateUserArgs {
        email: "alice@colomba.link",
        name: "Alice",
    });
    let key = Gpg::key_with_public_key(&mut gpg, &key.as_ref().expect("a key")).unwrap();

    (doc_dir, key)
}

#[allow(dead_code)]
pub fn create_test_env_with_test_gpg_key(test_data_path: String) -> (PathBuf, Key) {
    let key_dir = create_test_env(&test_data_path);
    let options = CopyOptions::new();
    fs_extra::dir::copy("test/key1/.key", &test_data_path, &options).unwrap();


    let mut gpg = Gpg::new();
    let key = gpg
        .get_public_key("A84E5D451E9E75B4791556896F45F34A926FBB70")
        .unwrap();

    if(false){
        // todo: move this somewhere else...

        let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
            .expect("Could create pgpme context from open pgp protocol");
        context.set_armor(true);
        let mut data: Vec<u8> = Vec::new();
       context
            .export_keys(&[key.public.clone().unwrap()], gpgme::ExportMode::empty(), &mut data)
            .expect("Could not export key");
        println!("{}", String::from_utf8(data).unwrap());

       context
            .set_key_list_mode(gpgme::KeyListMode::WITH_SECRET)
            .unwrap();
        let mut sec_data: Vec<u8> = Vec::new();
       context
            .export(
                Some(key.fingerprint.clone()),
                ExportMode::SECRET,
                &mut sec_data,
            )
            .unwrap();
        println!("{}", String::from_utf8(sec_data).unwrap());
    }


    (key_dir, key)
}
