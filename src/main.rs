#![warn(unused_extern_crates)]

use chrono::Local;
use clap::Parser;
use cli::{Args, Dataset, Storage};
use polars::{export::num::ToPrimitive, prelude::*};
use std::fs;
use vdb::{prelude::Result, storage, vector::generate_random_vectors, Node};
mod cli;

const MAX_NEIGHBOUR_COUNT: u8 = 5;

// 38461 vectors of 1536 dimensions
// in-mem indexing: 842.50ms
// disk indexing: 13.25s
// fresh-disk graph::index took 2.11s (before io_uring optimisation)
//
// syscalls count for fresh-disk (before io_uring optimisation)
// - 480680
//
// 1,000,000 vectors of dimension: 1536
// In-mem graph::new took 5.854s
// In-mem graph::index took 46.901s
//
// Disk graph::new took 194.129s
// Disk graph::index took 1635.210s
//
// fresh-disk graph::new took 85.325s 158.42s, 111.485s
// fresh-disk graph::index took 498.031s
fn main() {
    let args = Args::parse();

    match args.dataset {
        Dataset::Dbpedia => run_dataset_test(args.storage_type),
        Dataset::Debug => {
            debug(
                2000,
                std::ops::Range {
                    start: 2000.0,
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

fn run_dataset_test(storage_type: Storage) {
    let res = read_dataset("dataset/dbpedia-entities-openai-1M/data/", -1);
    // TODO: hardcode dimensions for now
    // println!("Read {} vectors of dimension: {}", res.len(), res[0].len());
    let storage = new_storage(storage_type, 1536 as u16, MAX_NEIGHBOUR_COUNT);
    let start = std::time::Instant::now();
    let mut graph = vdb::graph::Graph::new(res, 2, MAX_NEIGHBOUR_COUNT, storage).unwrap();
    println!("{:?} graph::new took {:?}", storage_type, start.elapsed());

    let start = std::time::Instant::now();
    graph.index(1.2).unwrap();
    println!("{:?} graph::index took {:?}", storage_type, start.elapsed());
}

fn read_datafile(file: &str) -> Vec<Vec<f32>> {
    let args = ScanArgsParquet::default();
    let df = LazyFrame::scan_parquet(file, args)
        .unwrap()
        .collect()
        .unwrap();
    let vector_column = df.column("openai").unwrap().list().unwrap();
    fn parse_vector(arr: Option<Series>) -> Vec<f32> {
        let series = arr.ok_or("series not found").unwrap();
        let arr_value = series.f64().unwrap();
        let single_vector: Vec<f32> = arr_value.into_iter().filter_map(|x| x?.to_f32()).collect();
        single_vector
    }
    let vecs = vector_column.into_iter().map(parse_vector);
    vecs.collect()
}

fn read_dataset(dataset_path: &str, ingest_files: i64) -> impl Iterator<Item = Vec<Vec<f32>>> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dataset_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().unwrap() == "parquet" {
            if ingest_files >= 0 && paths.len() == ingest_files as usize {
                break;
            }
            paths.push(path.to_str().unwrap().to_string());
        }
    }

    let vec_iter = paths
        .into_iter()
        .map(|path: String| -> _ { read_datafile(&path) });
    vec_iter
}

fn debug(
    seed_dataset_size: usize,
    vector_value_range: std::ops::Range<f32>,
    storage_type: Storage,
) {
    let test_vectors = generate_random_vectors(seed_dataset_size, &vector_value_range, 2);
    let storage = new_storage(
        storage_type,
        test_vectors[0].len() as u16,
        MAX_NEIGHBOUR_COUNT,
    );
    let mut graph = vdb::graph::Graph::new(
        vec![test_vectors].into_iter(),
        2,
        MAX_NEIGHBOUR_COUNT,
        storage,
    )
    .unwrap();
    let mut plotter = vdb::plotter::Plotter::new(vector_value_range.clone());

    // plot initial
    let nodes = graph.storage.get_all_nodes().unwrap();
    plotter.set_connected_nodes(&nodes);

    let (closests, _) = graph.greedy_search(1, &[1000.0f32, 1000.0f32], 3, 10);
    let closest_nodes: Vec<Node> = closests
        .iter()
        .filter_map(|&id| graph.storage.get_node(id).ok())
        .collect();
    plotter.set_isolated_nodes(&closest_nodes);

    let path = format!("static/{}/", Local::now().format("%Y-%m-%d"));
    fs::create_dir_all(&path).unwrap();
    plotter
        .plot(&format!("{}/graph-initial.png", path), "Initial graph")
        .unwrap();

    // plot alpha=1.0
    graph.index(1.0).unwrap();
    let (closests, _) = graph.greedy_search(1, &[1000.0f32, 1000.0f32], 3, 10);
    let closest_nodes: Vec<Node> = closests
        .iter()
        .filter_map(|&id| graph.storage.get_node(id).ok())
        .collect();
    plotter.set_isolated_nodes(&closest_nodes);
    plotter.set_connected_nodes(&graph.storage.get_all_nodes().unwrap());
    plotter
        .plot(&format!("{}/graph-1.png", path), "first pass, α=1")
        .unwrap();

    // alpha=1.2
    graph.index(1.2).unwrap();
    let (closests, _) = graph.greedy_search(1, &[1000.0f32, 1000.0f32], 3, 10);
    let closest_nodes: Vec<Node> = closests
        .iter()
        .filter_map(|&id| graph.storage.get_node(id).ok())
        .collect();
    plotter.set_isolated_nodes(&closest_nodes);
    plotter.set_connected_nodes(&graph.storage.get_all_nodes().unwrap());
    plotter
        .plot(&format!("{}/graph-2.png", path), "second pass, α=1.2")
        .unwrap();

    // insert new node
    let inserted_node = graph.insert(vec![1000.0, 1000.0], 1, 1.2, 10).unwrap();
    plotter.set_connected_nodes(&graph.storage.get_all_nodes().unwrap());
    plotter.set_isolated_nodes(&vec![inserted_node]);
    plotter
        .plot(&format!("{}/graph-3.png", path), "inserted")
        .unwrap();
}

fn new_storage(
    storage_type: Storage,
    dimensions: u16,
    max_neighbour_count: u8,
) -> Box<dyn storage::GraphStorage> {
    match storage_type {
        Storage::InMem => Box::new(storage::InMemStorage::new()),
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
