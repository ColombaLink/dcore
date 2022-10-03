//todo: add to a spare cli crate

use std::error::Error;
use std::os;
use std::path::PathBuf;
use clap::Parser;
use dcore::doc::{Doc, DocumentInitOptions, DocumentInitOptionsIdentity};
use dcore::Identity;

#[derive(clap::Parser)]
#[clap(author = "Fabrizio Parrillo <fabrizio.parrillo@colomba.link>", version = "v0.0.1")]

struct Args {
    #[clap(subcommand)]
    dcore_sub: DcoreSubCommands,
}

#[derive(clap::Parser)]
enum DcoreSubCommands {
    IdentityCreate(IdentityCreateArgs),
    IdentityListAll(IdentityListAllArgs),

    DocumentCreate(DocumentCreateArgs),

    ResourceListAll(ResourceListAllArgs),
}

fn main() {
    let options = Args::parse();
    let result = match options.dcore_sub {
        DcoreSubCommands::IdentityCreate(args) => identity_create(args),
        DcoreSubCommands::IdentityListAll(args) => identity_list_all(args),

        DcoreSubCommands::DocumentCreate(args) => document_create(args),

        DcoreSubCommands::ResourceListAll(args) => resource_create(args),
    };
}



/// Create a new identity
///
/// dcore identity-create --keyring-home ./gpghome
#[derive(clap::Parser)]
struct IdentityCreateArgs {

    /// keyring home directory
    /// default is ~/.dybli/keys
    #[clap(short, long)]
    keyring_home:  Option<String>,
}

fn identity_create(init_args: IdentityCreateArgs) -> Result<(), Box<dyn Error>> {
    println!("Create a new identity.");
    match Identity::create_identity(init_args.keyring_home) {
        Ok(_) => {println!("Created identity.")},
        Err(e) => { print!("{}", e);}
    };

    Ok(())
}

/// List all identities
///
/// dcore identity-list-all --keyring-home ./gpghome
#[derive(clap::Parser)]
struct IdentityListAllArgs {

    /// keyring home directory
    /// default is ~/.dybli/keys
    #[clap(short, long)]
    keyring_home:  Option<String>,
}

fn identity_list_all(args: IdentityListAllArgs) -> Result<(), Box<dyn Error>> {
    println!("List all identities.");
    match Identity::print_all_identities(args.keyring_home) {
        Ok(_) => {},
        Err(e) => { print!("{}", e);}
    };

    Ok(())
}




/// Create a document
///
/// The document is created in the current directory.
/// The provided document name is used as the directory name.
///
/// dcore document-create --keyring-home ./gpghome
#[derive(clap::Parser)]
struct DocumentCreateArgs {

    /// keyring home directory
    /// default is ~/.dybli/keys
    #[clap(short, long)]
    keyring_home:  Option<String>,

    /// Document name
    #[clap(short, long)]
    document_name: String,

    /// User identity fingerprint
    #[clap(short, long)]
    user_id_fingerprint: String,

}

fn document_create(args: DocumentCreateArgs) -> Result<(), Box<dyn Error>> {
    println!("Create a new document..");
    // 1. Get the identity by the fingerprint

    let identity = Identity::get_identity(dcore::identity::GetIdentityArgs {
        keyring_home_dir: args.keyring_home,
        fingerprint: args.user_id_fingerprint
    }).expect("Failed to get identity with the provided fingerprint");

    // 2. Create the document, and the config resource in the document.
    //    The config resource contains the document name and the user identity fingerprint + public key.

    std::fs::create_dir(&args.document_name).expect("Failed to create document directory");

    let docInitOptions = DocumentInitOptions {
        directory: PathBuf::from(args.document_name),
        identity: DocumentInitOptionsIdentity {
            public_key: identity.fingerprint.clone(), // todo: map the key object to the public key string
            fingerprint: identity.fingerprint.clone(),
        }
    };
    let document = Doc::init(&docInitOptions).expect("Failed to create document");
    Ok(())
}






/// List all identities
///
/// dcore resource-list-all --keyring-home ./gpghome
#[derive(clap::Parser)]
struct ResourceListAllArgs {

    /// keyring home directory
    /// default is ~/.dybli/keys
    #[clap(short, long)]
    keyring_home:  Option<String>,

}

fn resource_list_all(args: ResourceListAllArgs) -> Result<(), Box<dyn Error>> {
    println!("List all identities.");
    match Resource::print_all_identities(args.keyring_home) {
        Ok(_) => {},
        Err(e) => { print!("{}", e);}
    };

    Ok(())
}

