#![allow(unused)]

use std::error::Error;
use std::path::PathBuf;
use clap::Parser;
use dcore::doc::{ Doc, DocumentInitOptions};

#[derive(clap::Parser)]
#[clap(author = "Fabrizio Parrillo <fabrizio.parrillo@colomba.link>", version = "v0.0.1")]
struct Args {
    #[clap(subcommand)]
    sub_cmd: SubCommands,
}

#[derive(clap::Parser)]
enum SubCommands {
    Init(Init)
}


/// Init a Dybli Document
#[derive(clap::Parser)]
struct Init {

    /// name
    #[clap(short, long)]
    path:  PathBuf,

    /// name
    #[clap(short, long)]
    name:  Option<String>,

}

fn main() {
    let options = Args::parse();
    let result = match options.sub_cmd {
        SubCommands::Init(init) => init_document(init),
    };
}

fn init_document(init_args: Init) -> Result<Option<String>, Box<dyn Error>> {
    println!("Create dybli document.");
    let init_args = DocumentInitOptions {directory: init_args.path};

    match Doc::init(&init_args) {
        Ok(_) => {println!("Created document.")}
        Err(e) => { print!("{}", e.message());}
    }

    Ok(None)

}
