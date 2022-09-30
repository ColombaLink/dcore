//todo: add to a spare cli crate

use std::error::Error;
use std::path::PathBuf;
use clap::Parser;
use dcore::doc::{ Doc, DocumentInitOptions};
use dcore::Identity;

#[derive(clap::Parser)]
#[clap(author = "Fabrizio Parrillo <fabrizio.parrillo@colomba.link>", version = "v0.0.1")]

struct Args {
    #[clap(subcommand)]
    dcore_sub: DcoreSubCommands,
}

#[derive(clap::Parser)]
enum DcoreSubCommands {
    DocInit(InitDocArgs),
    IdentityCreate(IdentityCreateArgs),
    IdentityListAll(IdentityListAllArgs),
}

fn main() {
    let options = Args::parse();
    let result = match options.dcore_sub {
        DcoreSubCommands::DocInit(args) => document_init(args),

        DcoreSubCommands::IdentityCreate(args) => identity_create(args),
        DcoreSubCommands::IdentityListAll(args) => identity_list_all(args),
    };
}


/// Init a Dybli Document
#[derive(clap::Parser)]
struct InitDocArgs {

    /// name
    #[clap(short, long)]
    path:  PathBuf,

    /// name
    #[clap(short, long)]
    name:  Option<String>,

}

fn document_init(init_args: InitDocArgs) -> Result<Option<String>, Box<dyn Error>> {
    println!("Create dybli document.");
    let init_args = DocumentInitOptions {directory: init_args.path};

    match Doc::init(&init_args) {
        Ok(_) => {println!("Created document.")}
        Err(e) => { print!("{}", e.message());}
    }

    Ok(None)
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

fn identity_create(init_args: IdentityCreateArgs) -> Result<Option<String>, Box<dyn Error>> {
    println!("Create a new identity.");
    match Identity::create_identity(init_args.keyring_home) {
        Ok(_) => {println!("Created identity.")},
        Err(e) => { print!("{}", e);}
    };

    Ok(None)
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

fn identity_list_all(args: IdentityListAllArgs) -> Result<Option<String>, Box<dyn Error>> {
    println!("List all identities.");
    match Identity::print_all_identities(args.keyring_home) {
        Ok(_) => {},
        Err(e) => { print!("{}", e);}
    };

    Ok(None)
}



