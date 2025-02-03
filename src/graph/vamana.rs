use std::fs;

use chrono::Local;

use crate::{
    constant::{MAX_NEIGHBOUR_COUNT, SEED_DATASET_SIZE, VECTOR_DIMENSION, VECTOR_VALUE_RANGE},
    graph::Node,
    storage::InMemStorage,
};

use super::{graph::Graph, plotter::Plotter, vector::generate_random_vectors};
use crate::storage::NaiveDisk;

pub fn init() {
    let test_vectors = generate_random_vectors(SEED_DATASET_SIZE as usize, VECTOR_VALUE_RANGE, 2);

    let disk = NaiveDisk::new(
        VECTOR_DIMENSION,
        MAX_NEIGHBOUR_COUNT,
        "disk.index",
        "disk.free",
    )
    .unwrap();

    let in_mem = InMemStorage::new();
    let mut graph = Graph::new(&test_vectors, 2, MAX_NEIGHBOUR_COUNT, Box::new(disk)).unwrap();
    let mut plotter = Plotter::new();

    let nodes = graph.storage.get_all_nodes();
    plotter.set_connected_nodes(&nodes);

    let (closests, _) = graph.greedy_search(0, &[1000.0f32, 1000.0f32], 3, 10);
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

    // alpha=1
    graph.index(1.0).unwrap();
    let (closests, _) = graph.greedy_search(0, &[1000.0f32, 1000.0f32], 3, 10);
    let closest_nodes: Vec<Node> = closests
        .iter()
        .filter_map(|&id| graph.storage.get_node(id).ok())
        .collect();
    plotter.set_isolated_nodes(&closest_nodes);
    plotter.set_connected_nodes(&graph.storage.get_all_nodes());
    plotter
        .plot(&format!("{}/graph-1.png", path), "first pass, α=1")
        .unwrap();

    // alpha=1.2
    graph.index(1.2).unwrap();
    let (closests, _) = graph.greedy_search(0, &[1000.0f32, 1000.0f32], 3, 10);
    let closest_nodes: Vec<Node> = closests
        .iter()
        .filter_map(|&id| graph.storage.get_node(id).ok())
        .collect();
    plotter.set_isolated_nodes(&closest_nodes);
    plotter.set_connected_nodes(&graph.storage.get_all_nodes());
    plotter
        .plot(&format!("{}/graph-2.png", path), "second pass, α=1.2")
        .unwrap();
    // disk::write_to_disk(&graph);
}
