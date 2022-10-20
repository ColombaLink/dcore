pub use crate::document::{Document, DocumentInitOptions};
pub use crate::identity::Identity;

pub mod document;
mod gpg;
mod errors;
pub mod identity;
mod resource;
mod event;
mod document_utils;
mod libp2p_sync;

#[cfg(test)]
mod test_utils;
