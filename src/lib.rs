pub mod doc;
mod gpg;
mod errors;

pub use crate::doc::{Doc, DocumentInitOptions};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
