mod disk;
mod inmem;
mod storage;

pub use disk::NaiveDisk;
pub use inmem::InMemStorage;
pub use storage::GraphStorage;
