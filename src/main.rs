#![warn(unused_extern_crates)]

use chrono::Local;
use clap::Parser;
use cli::{Args, Dataset, Storage};
use std::fs;
use vdb::{storage, vector::generate_random_vectors, InMemStorage, Node};

mod cli;
mod data;

const MAX_NEIGHBOUR_COUNT: u8 = 5;
const DBPEDIA_DIMENSIONS: usize = 1536;

fn main() {
    let args = Args::parse();

    match args.dataset {
        Dataset::Dbpedia => {
            let graph = index_dbpedia(args.storage_type, 1);

            let test_query_vec: [f32; DBPEDIA_DIMENSIONS] = data::read_query_vector()
                .expect("Failed to read query vector")
                .as_slice()
                .try_into()
                .expect("Failed to convert slice to array");

            let similar_docs = query_dbpedia_index(&graph, &test_query_vec, 5);
            for doc in similar_docs {
                println!("{}\n", doc);
            }
        }
        Dataset::Debug => {
            debug(
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

/// index_dbpedia indexes the dbpedia dataset using index_storage_type. The number of files to read from the dataset can be specified with dataset_files. -1 to load all files (note that this will incur a huge indexing time)
fn index_dbpedia(index_storage_type: Storage, dataset_files: i64) -> vdb::Graph {
    let res = data::read_dataset("dataset/dbpedia-entities-openai-1M/data/", dataset_files);
    let storage = new_index_storage(
        index_storage_type,
        DBPEDIA_DIMENSIONS as u16,
        MAX_NEIGHBOUR_COUNT,
    );
    let start = std::time::Instant::now();
    let mut graph = vdb::graph::Graph::new(
        res,
        5,
        MAX_NEIGHBOUR_COUNT,
        storage,
        Box::new(InMemStorage::default()), // TODO: Can provide other implementations
    )
    .unwrap();
    println!(
        "{:?} graph::new took {:?}",
        index_storage_type,
        start.elapsed()
    );

    let start = std::time::Instant::now();
    graph.index(1.0).unwrap();
    graph.index(1.2).unwrap();
    println!(
        "{:?} graph::index took {:?}",
        index_storage_type,
        start.elapsed()
    );
    graph
}

fn query_dbpedia_index(graph: &vdb::Graph, query: &[f32], k: usize) -> Vec<String> {
    let closest_k_nodes = graph.greedy_search_random_start(query, k, 10);
    let mut result = Vec::with_capacity(closest_k_nodes.0.len());
    for node_index in closest_k_nodes.0 {
        let data = graph.data_store.get_data(node_index);
        if let Some(content) = data {
            result.push(content);
        }
    }
    result
}

fn debug(
    seed_dataset_size: usize,
    vector_value_range: std::ops::Range<f32>,
    storage_type: Storage,
) {
    let test_vectors = generate_random_vectors(seed_dataset_size, &vector_value_range, 2);
    let storage = new_index_storage(
        storage_type,
        test_vectors[0].0.len() as u16,
        MAX_NEIGHBOUR_COUNT,
    );

    let mut graph = vdb::graph::Graph::new(
        vec![test_vectors].into_iter(),
        3,
        MAX_NEIGHBOUR_COUNT,
        storage,
        Box::new(InMemStorage::default()),
    )
    .unwrap();
    let mut plotter = vdb::plotter::Plotter::new(vector_value_range.clone());

    // plot initial
    let nodes = graph.index_store.get_all_nodes().unwrap();
    plotter.set_connected_nodes(&nodes);

    let (closests, _) = graph.greedy_search(1, &[1000.0f32, 1000.0f32], 5, 10);
    let closest_nodes: Vec<Node> = closests
        .iter()
        .filter_map(|&id| graph.index_store.get_node(id).ok())
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
        .filter_map(|&id| graph.index_store.get_node(id).ok())
        .collect();
    plotter.set_isolated_nodes(&closest_nodes);
    plotter.set_connected_nodes(&graph.index_store.get_all_nodes().unwrap());
    plotter
        .plot(&format!("{}/graph-1.png", path), "first pass, α=1")
        .unwrap();

    // alpha=1.2
    graph.index(1.2).unwrap();
    let (closests, _) = graph.greedy_search(1, &[1000.0f32, 1000.0f32], 3, 10);
    let closest_nodes: Vec<Node> = closests
        .iter()
        .filter_map(|&id| graph.index_store.get_node(id).ok())
        .collect();
    plotter.set_isolated_nodes(&closest_nodes);
    plotter.set_connected_nodes(&graph.index_store.get_all_nodes().unwrap());
    plotter
        .plot(&format!("{}/graph-2.png", path), "second pass, α=1.2")
        .unwrap();

    // insert new node
    let inserted_node = graph
        .insert(vec![1000.0, 1000.0], "".to_string(), 1, 1.2, 10)
        .unwrap();
    plotter.set_connected_nodes(&graph.index_store.get_all_nodes().unwrap());
    plotter.set_isolated_nodes(&vec![inserted_node]);
    plotter
        .plot(&format!("{}/graph-3.png", path), "inserted")
        .unwrap();
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
