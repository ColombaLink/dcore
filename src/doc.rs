use git2::{Error, Repository, RepositoryInitOptions};
use std::path::{PathBuf};


pub struct Doc {
}

unsafe impl Send for Doc {}

pub struct DocumentInitOptions {
    pub directory: PathBuf,
}

impl Doc {

    pub fn init(args: &DocumentInitOptions) -> Result<(), Error> {
        let path = &args.directory;
        let mut init_options = RepositoryInitOptions::new();
        init_options.bare(true);
        Repository::init_opts(&path, &init_options)?;
        Ok(())
    }
}
