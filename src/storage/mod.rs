mod disk;
mod fresh_disk;
mod inmem;
mod storage;

pub use disk::NaiveDisk;
pub use fresh_disk::FreshDisk;
pub use inmem::InMemStorage;
pub use storage::{DataStore, GraphStorage};
