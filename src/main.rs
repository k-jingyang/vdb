#![warn(unused_extern_crates)]

use clap::Parser;
use cli::{Args, Dataset, Storage};
use vdb::storage;

mod cli;
mod data;
mod dbpedia;
mod debug;

const MAX_NEIGHBOUR_COUNT: u8 = 5;
const DBPEDIA_DIMENSIONS: usize = 1536;

fn main() {
    let args = Args::parse();

    match args.dataset {
        Dataset::Dbpedia => {
            let storage = new_index_storage(
                args.storage_type,
                DBPEDIA_DIMENSIONS as u16,
                MAX_NEIGHBOUR_COUNT,
            );
            let graph = dbpedia::index_dbpedia(storage, -1);

            let test_query_vec: [f32; DBPEDIA_DIMENSIONS] = data::read_query_vector()
                .expect("Failed to read query vector")
                .as_slice()
                .try_into()
                .expect("Failed to convert slice to array");

            let similar_docs = dbpedia::query_dbpedia_index(&graph, &test_query_vec, 5);
            for doc in similar_docs {
                println!("{}\n", doc);
            }
        }
        Dataset::Debug => {
            debug::debug(
                200,
                std::ops::Range {
                    start: 0.0,
                    end: 2000.0,
                },
                args.storage_type,
            );

            if args.storage_type == Storage::FreshDisk {
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
        }
    }
}

fn new_index_storage(
    storage_type: Storage,
    dimensions: u16,
    max_neighbour_count: u8,
) -> Box<dyn storage::IndexStore> {
    match storage_type {
        Storage::InMem => Box::new(storage::InMemStorage::default()),
        Storage::PureDisk => Box::new(
            storage::NaiveDisk::new(dimensions, max_neighbour_count, "disk.index", "disk.free")
                .unwrap(),
        ),
        Storage::FreshDisk => Box::new(
            storage::FreshDisk::new(dimensions, max_neighbour_count, "disk.index", "disk.free")
                .unwrap(),
        ),
    }
}
