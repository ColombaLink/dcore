use fs_extra::dir::CopyOptions;
use std::fs;
use std::path::PathBuf;

use gpgme::ExportMode;
use sequoia_openpgp::parse::Parse;
use sequoia_openpgp::serialize::MarshalInto;
use sequoia_openpgp::Cert;
use sequoia_openpgp::crypto::mpi::SecretKeyMaterial;

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


#[allow(dead_code)]
pub struct TestRSAKey {

    //  store data for 1024-bit RSA key.
    pub public_key: String,
    pub secret_key: String,
    pub passphrase: String,
}

// keys generated from https://pgptool.org with 1024-bit RSA key.
#[allow(dead_code)]
pub fn get_rsa_test_key() -> TestRSAKey {
    TestRSAKey {
        passphrase: "12345".to_string(),
        public_key: r#"-----BEGIN PGP PUBLIC KEY BLOCK-----
Version: Keybase OpenPGP v2.0.76
Comment: https://keybase.io/crypto

xo0EY1K9ZAEEANYN4OGgST7Hhtp4Hbjhy7v4ssdkygkKgoGyrz8fKgrX2gzIDSzs
8nrZQTMvaDqpG1DJ4OyE+psZJvtBMAGjq+9f5h9oNOQzI2Cx5Xbs2n+3EOosU/hh
u6UKyvvjA6XfhvaYEZo3sat6gsf7x9yyRZEuBg34LSK1n9z7gWv1lsuLABEBAAHN
F0FsaWNlIDxhbGljZUBnbWFpbC5jb20+wroEEwEKACQFAmNSvWQCGy8DCwkHAxUK
CAIeAQIXgAMWAgECGQEFCQAAAAAACgkQv5MrH0nvbaq3igP/SMZ+r/x3i0bS5ZLE
3paFTsaFpOEWuY4YLVGrVYpgOZzOb2a/zM/UuscbhtNj3Exgn85v5J/4nckv7X7/
r+cnV25C5MyZo62DCA9y2e3kiJrnd32E2wi2gUCExPDl1CWp9WKyjO1StTSN3YH4
W0JE6/QD8VWEwfstI1tWf5ut8//OjQRjUr1kAQQAsEXsvskZR4J9BKVVdn4MPkBv
duNxEPwMf6+2gyRIIJ4Y2mykw55dH+6EUnneh3etazNrr22kmr/bxYeEfrm6FkSF
MihmWREzpq4kn374ldn3WcG8pvDmsXFxrhekL8cYpxVf1lapAoM66/vY8qVHltw0
4f2CjBqERgJ5Alfg/60AEQEAAcLAgwQYAQoADwUCY1K9ZAUJAAAAAAIbLgCoCRC/
kysfSe9tqp0gBBkBCgAGBQJjUr1kAAoJEAhj0mXqRKtlUFED+wVaklFjz31x4bE+
VGMDW6ygncx9Xu7GTK466kq1W7FacF8VHIx1Dt7LzIPy0MhEZd2hI9zyhhYz3awd
oDx+K4YVXpuA8nIOPbwaf2cQEt563BdR/u1X3zgAHkfRd36qQwldgrt562NQAAAk
CjauoV8Y/7mfzEu67jAVhCPW8X6rxw8D/1rs78SXtIj1byHeCnucVJHxkDp2Cxmd
aE81yaOpX6LM8lYeg0EC9+x1AkNF4z9tpBOW4O8i7rCeVVsP8/I2ohmf2GiXggPt
Jd+sBr4J6ahXkMEAN5UfID/vhMwm6cKRPM9YfSfZH0ztfYS2qewVAvd4M31hKmo3
viuAwvrTUrc7zo0EY1K9ZAEEANDxF2+vVly0X/f0t5mfN08c8ysAz5VWSC1RCKqj
wVS7JboVxyuu0GQGdgPBhlx4cI+e5ykDUhhDE2Or27Ii7FQ3WolKLvz9vcT8Yd/Z
Tc7WsIHUCas8FbmO2SAG/ie++9vKbTwwwGm2X8JilDjDSijXLiz8CZ47INjWneXL
hpa1ABEBAAHCwIMEGAEKAA8FAmNSvWQFCQAAAAACGy4AqAkQv5MrH0nvbaqdIAQZ
AQoABgUCY1K9ZAAKCRCSFRC7nCsaPqhrA/9QmhfEySdhji+1kLxubbItJYPY+njX
gN44ruo8Ap0UVf3+Z5flI4mr2LkL5OGTm8gh4kaKYZxxYHboNb/sQ4ff4zq+0xNB
uRVCZFm10OJaGmOeYp5m6gjMhjUfYXpQClMdtdlM46NVu40EbXYufLDGmCEdNRZl
JZ2aNO65aENr9KJKBADN43cnwRUvB3m3FFumdECagcddur8Ux8uu/gI0P5W3Coy9
z4Ur2WBmfcYbEd0vK44g2qbxT9PCLZ3Ke7UuPXNep9Nh4yB9eGeRLuYbhR/I1J5g
DmMWFRgtmBySS1sNwpqjFOVlDCpjkOGnRvupbtVbUFowFj48T8390LK89V4BAg==
=TdMe
-----END PGP PUBLIC KEY BLOCK-----
"#.to_string(),

        secret_key: r#"-----BEGIN PGP PRIVATE KEY BLOCK-----
Version: Keybase OpenPGP v2.0.76
Comment: https://keybase.io/crypto

xcFGBGNSvWQBBADWDeDhoEk+x4baeB244cu7+LLHZMoJCoKBsq8/HyoK19oMyA0s
7PJ62UEzL2g6qRtQyeDshPqbGSb7QTABo6vvX+YfaDTkMyNgseV27Np/txDqLFP4
YbulCsr74wOl34b2mBGaN7GreoLH+8fcskWRLgYN+C0itZ/c+4Fr9ZbLiwARAQAB
/gkDCH+qR4XikmXrYNghpyHofr/lbFl5sfv+5Zr1i54opua0RhcpMH+hCyvYiXhM
+7day1i7zu30s8lheKZlhF+eGhMCvDu7fELOBzyNCXbxURaoZQGujNxJ3T1Bq9L5
ZQxGzsBMghAfkKeEoJgo79kdRxA5z1Tb/3JT+vfmgUZUoH9FCa2rBx7pNx+k4Cbi
GBz3NZ8uT+a9XhPRLIBj+gim+297Hj3HDFn6JcAOAUmkAi7BdmcgMM/CDcJBXmaW
80F9bYhUc92gI3GbIeuOxmM8hNdG975C+11ooEqbsXk21myEJcy3GKsQN6nOCz+Q
HKd6ClVOLzrBANqWeM9IQMTC1r5KgU/vWH+Kucz1JrIXqpNs+/DFSz8yAO3sZabS
ourjxGVbV1PhFtGCtfEZwKJTrYOw+B7+MHeymcpTZSPPy29Lc29vdK3KZ8N3NA1a
V/CBn65Eed8DIkqkHPbtwCoVZZ61ds1pW6y107wTxHh1tvgNDIgoqATNF0FsaWNl
IDxhbGljZUBnbWFpbC5jb20+wroEEwEKACQFAmNSvWQCGy8DCwkHAxUKCAIeAQIX
gAMWAgECGQEFCQAAAAAACgkQv5MrH0nvbaq3igP/SMZ+r/x3i0bS5ZLE3paFTsaF
pOEWuY4YLVGrVYpgOZzOb2a/zM/UuscbhtNj3Exgn85v5J/4nckv7X7/r+cnV25C
5MyZo62DCA9y2e3kiJrnd32E2wi2gUCExPDl1CWp9WKyjO1StTSN3YH4W0JE6/QD
8VWEwfstI1tWf5ut8//HwUYEY1K9ZAEEALBF7L7JGUeCfQSlVXZ+DD5Ab3bjcRD8
DH+vtoMkSCCeGNpspMOeXR/uhFJ53od3rWsza69tpJq/28WHhH65uhZEhTIoZlkR
M6auJJ9++JXZ91nBvKbw5rFxca4XpC/HGKcVX9ZWqQKDOuv72PKlR5bcNOH9gowa
hEYCeQJX4P+tABEBAAH+CQMIU8Q9ZgDpG6Fg4KZao/1IStSjlV0AyD32/efz4FrP
5NST2mPgQc+mF+ewp5IGOVRROVbHgbN89+rluoQkutumYjkhHtAO8n9aVhgtEY4t
tHaeWX83IkEYLAeffm5v5agaFBMIi9JFs2HmMV8pwufq3DHxpE+TZVhZx4DMOZuU
dSoSlLmCMIn5+LRXSP7hFE6nMz4HLZIUf1Z/f1DNjbuwcdCrG05VcBjdyrRH5avi
KxmjRvYCtkq4XhYIomAty3ewDTOCnFj2Z8Jo/haY1oVNWBndYMBMfAfuJfPe6/j5
RboJTRw5wjyzx6Y83iwlN3+2GLNeWR4SG+ixPRjOi3OXvqS6xSl/Vk9AvhNG7wZQ
auaQEitbPtG4T5U+lZx7m30CX3ryjobK1m0lIKMPDetWYz64dUOc5E7YN87hu6RC
3MggfxCvRvCGcC6dYGxMzo+GtbtRYkGkacF36Sn0jp/Uy5XwA2zZoeVHuJVAPzjG
2kHDOgCqP8LAgwQYAQoADwUCY1K9ZAUJAAAAAAIbLgCoCRC/kysfSe9tqp0gBBkB
CgAGBQJjUr1kAAoJEAhj0mXqRKtlUFED+wVaklFjz31x4bE+VGMDW6ygncx9Xu7G
TK466kq1W7FacF8VHIx1Dt7LzIPy0MhEZd2hI9zyhhYz3awdoDx+K4YVXpuA8nIO
Pbwaf2cQEt563BdR/u1X3zgAHkfRd36qQwldgrt562NQAAAkCjauoV8Y/7mfzEu6
7jAVhCPW8X6rxw8D/1rs78SXtIj1byHeCnucVJHxkDp2CxmdaE81yaOpX6LM8lYe
g0EC9+x1AkNF4z9tpBOW4O8i7rCeVVsP8/I2ohmf2GiXggPtJd+sBr4J6ahXkMEA
N5UfID/vhMwm6cKRPM9YfSfZH0ztfYS2qewVAvd4M31hKmo3viuAwvrTUrc7x8FG
BGNSvWQBBADQ8Rdvr1ZctF/39LeZnzdPHPMrAM+VVkgtUQiqo8FUuyW6FccrrtBk
BnYDwYZceHCPnucpA1IYQxNjq9uyIuxUN1qJSi78/b3E/GHf2U3O1rCB1AmrPBW5
jtkgBv4nvvvbym08MMBptl/CYpQ4w0oo1y4s/AmeOyDY1p3ly4aWtQARAQAB/gkD
COdjE6/c83UmYCuVHBj/U7TCCifiGwuia7Bo7ZLVqMRIWuyxlZSl5yz2UOEZ7hV7
Y6eEDGWAxdvmsihK+GqMyYbIlNEQzkgqkFQIjjbsfM+1CteBxW0gcGghuvX0UEHX
fSetynncSYEH+dKw0fQKvPWjhptbVA3HXhw2Ebgr1LKo0C2YrUHJqmLNOWbIzmiz
JjPyvSkKVnVx8/WaXJo6rfPN+Fq4ueI4qHk0fT/SIEviRBHAs9F7+ca7cenHD21Y
Ruh2dbgmMHoTST5RXnsD2okeTOV7xuaDHqxMlIlC8ApBLiZCL+F2iWd/FT6jtEZG
QAsjoPR+sI+Xy0Fdqx+jW2WH5MaLFAPadp7P/tL95psHmhVwFOONAOIBtrb2qWlk
IEJpYY7lHWKLrOcYgZ1+yzOaBfQ5EFmu3bX98G3lCyk1pGVeqfv6VvgRcQFUVWE8
G5kzFaRFsCmeN3haTzajavk3LBbJQT18B9bBw4Vb3ZcF10uAM9DCwIMEGAEKAA8F
AmNSvWQFCQAAAAACGy4AqAkQv5MrH0nvbaqdIAQZAQoABgUCY1K9ZAAKCRCSFRC7
nCsaPqhrA/9QmhfEySdhji+1kLxubbItJYPY+njXgN44ruo8Ap0UVf3+Z5flI4mr
2LkL5OGTm8gh4kaKYZxxYHboNb/sQ4ff4zq+0xNBuRVCZFm10OJaGmOeYp5m6gjM
hjUfYXpQClMdtdlM46NVu40EbXYufLDGmCEdNRZlJZ2aNO65aENr9KJKBADN43cn
wRUvB3m3FFumdECagcddur8Ux8uu/gI0P5W3Coy9z4Ur2WBmfcYbEd0vK44g2qbx
T9PCLZ3Ke7UuPXNep9Nh4yB9eGeRLuYbhR/I1J5gDmMWFRgtmBySS1sNwpqjFOVl
DCpjkOGnRvupbtVbUFowFj48T8390LK89V4BAg==
=m/C/
-----END PGP PRIVATE KEY BLOCK-----
"#.to_string(),
    }
}

// test sequioa with RSA key
#[allow(dead_code)]
pub fn rsa_key() -> () {

    let mut context = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
        .expect("Could create pgpme context from open pgp protocol");
    context.set_armor(true);

    let key = get_rsa_test_key();
    let mut sec_armoured_key = Cert::from_bytes(key.secret_key.as_bytes());
    let sec_key = sec_armoured_key.unwrap().primary_key().key().clone().parts_into_secret();
    let mut sec_key_decrypted = sec_key.unwrap().decrypt_secret(&key.passphrase.into()).unwrap().into_keypair().unwrap().secret().clone();

    sec_key_decrypted.map(|pz|{

        // seems sequoia store coefficients for RSA
        // https://docs.sequoia-pgp.org/sequoia_openpgp/crypto/mpi/enum.SecretKeyMaterial.html
        let D ;
        let P;
        let Q;
        let U;
        let key_algo = pz.algo().unwrap().clone(); // RSA!!!

        match pz {
            SecretKeyMaterial::RSA { d,p,q,u } => {
                D = d.value().to_vec().clone();
                P = p.value().to_vec().clone();
                Q = q.value().to_vec().clone();
                U = u.value().to_vec().clone();
                println!("got RSA coefficients")}
            SecretKeyMaterial::DSA { .. } => {}
            SecretKeyMaterial::ElGamal { .. } => {}
            SecretKeyMaterial::EdDSA { .. } => {}
            SecretKeyMaterial::ECDSA { .. } => {}
            SecretKeyMaterial::ECDH { .. } => {}
            SecretKeyMaterial::Unknown { .. } => {}
            _ => {}
        }

    });

    let tmp = 0;
    println!("done {}", tmp);

}


