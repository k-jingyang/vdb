use clap::{Parser, ValueEnum};

/// Run toy implementation of DiskANN
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Storage type to use
    #[arg(value_enum)]
    pub(crate) storage_type: Storage,

    /// Type of dataset to run test with
    #[arg(value_enum)]
    pub(crate) dataset: Dataset,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq)]
pub(crate) enum Storage {
    /// In-mem
    InMem = 0,
    /// Pure disk
    PureDisk = 1,
    /// FreshDiskANN
    FreshDisk = 2,
}

#[derive(Copy, Clone, ValueEnum)]
pub(crate) enum Dataset {
    /// dbpedia-entities-openai-1M, 1 million vectors of 1536 dimensions
    Dbpedia,
    /// randomly generated 2 thousand vectors of 2 dimensions. Visual graphs plotted under static/${date}
    Debug,
}
