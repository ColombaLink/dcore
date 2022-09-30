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
}

fn main() {
    let options = Args::parse();
    let result = match options.dcore_sub {
        DcoreSubCommands::DocInit(init) => document_init(init),
        DcoreSubCommands::IdentityCreate(init) => identity_create(init),
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


