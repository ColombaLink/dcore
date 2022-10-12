//todo: add to a spare cli crate

use std::error::Error;
use std::os;
use std::path::PathBuf;
use clap::Parser;
use dcore::document::{Document, DocumentInitOptions, DocumentInitOptionsIdentity, DocumentNewOptions};
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
    ResourceCat(ResourceCatArgs),
}

fn main() {
    let options = Args::parse();
    let result = match options.dcore_sub {
        DcoreSubCommands::IdentityCreate(args) => identity_create(args),
        DcoreSubCommands::IdentityListAll(args) => identity_list_all(args),

        DcoreSubCommands::DocumentCreate(args) => document_create(args),

        DcoreSubCommands::ResourceListAll(args) => resource_list_all(args),
        DcoreSubCommands::ResourceCat(args) => resource_cat(args),

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

    // user name
    #[clap(short, long)]
    name:  String,

    // user email
    #[clap(short, long)]
    email:  String,

}

fn identity_create(init_args: IdentityCreateArgs) -> Result<(), Box<dyn Error>> {
    println!("Create a new identity.");
    match Identity::create_identity(init_args.keyring_home, &init_args.name, &init_args.email) {
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

    let docInitOptions = DocumentNewOptions {
        directory: PathBuf::from(&args.document_name),
        name: args.document_name.clone(),
        identity_fingerprint: identity.fingerprint.clone(),
    };
    let mut doc = Document::new(docInitOptions).expect("Failed to create document");

    let public_key = {
        let mut gpg = &mut doc.gpg;
        let public_key = gpg.get_public_key_by_identity(&doc.identity).expect("Failed to get public key by identity");
        String::from_utf8(public_key).expect("Failed to convert public key to string")
    };
    println!("Public key: {}", public_key);

    let document = doc.init(&identity.fingerprint, &public_key).expect("Failed to create document");
    Ok(())
}






/// List all resources of a document
///
/// dcore resource-list-all --keyring-home ./gpghome
#[derive(clap::Parser)]
struct ResourceListAllArgs {

    /// keyring home directory
    /// default is ~/.dybli/keys
    #[clap(short, long)]
    keyring_home:  Option<String>,

    /// Path to the document directory
    #[clap(short, long)]
    document_path: String,


    /// User identity fingerprint
    #[clap(short, long)]
    user_id_fingerprint: String,
}

fn resource_list_all(args: ResourceListAllArgs) -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(&args.document_path);
    let name = directory.file_name().unwrap().to_str().unwrap().to_string();
    println!("List all resources of document with name:  {}.", &name);

    let identity = Identity::get_identity(dcore::identity::GetIdentityArgs {
        keyring_home_dir: args.keyring_home,
        fingerprint: args.user_id_fingerprint
    }).expect("Failed to get identity with the provided fingerprint");

    let docInitOptions = DocumentNewOptions {
        directory,
        name,
        identity_fingerprint: identity.fingerprint.clone(),
    };

    // todo: we need to be able to load the doc without the identity
    //       for the case that a user just want to list them without... makes only sense for unencrypted docs...
    let mut doc = Document::new(docInitOptions).expect("Failed to create document");
    doc.load().expect("Failed to load document");

    println!("Resources:");
    doc.resources.iter().for_each(|(name, resource)| {
        println!("\t- {}", name);
    });

    Ok(())
}


/// Cat the current content of  document
///
/// dcore resource-cat
#[derive(clap::Parser)]
struct ResourceCatArgs {

    /// keyring home directory
    /// default is ~/.dybli/keys
    #[clap(short, long)]
    keyring_home:  Option<String>,

    /// User identity fingerprint
    #[clap(short, long)]
    user_id_fingerprint: String,

    /// Path to the document directory
    #[clap(short, long)]
    document_path: String,

    /// Name of the resource
    #[clap(short, long)]
    resource_name: String,
}

fn resource_cat(args: ResourceCatArgs) -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(&args.document_path);
    let name = directory.file_name().unwrap().to_str().unwrap().to_string();
    println!("List all resources of document with name:  {}.", &name);

    let identity = Identity::get_identity(dcore::identity::GetIdentityArgs {
        keyring_home_dir: args.keyring_home,
        fingerprint: args.user_id_fingerprint
    }).expect("Failed to get identity with the provided fingerprint");

    let docInitOptions = DocumentNewOptions {
        directory,
        name,
        identity_fingerprint: identity.fingerprint.clone(),
    };

    // todo: we need to be able to load the doc without the identity
    //       for the case that a user just want to list them without... makes only sense for unencrypted docs...
    let mut doc = Document::new(docInitOptions).expect("Failed to create document");
    doc.load().expect("Failed to load document");

    println!("Resource Content:");

    let resource =  doc.resources.get(&args.resource_name).expect("Resource not found");

    let content = resource.get_content();
    println!("{}", content);
    Ok(())
}
