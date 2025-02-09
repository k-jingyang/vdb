use std::fs;

use chrono::Local;

use crate::{constant::MAX_NEIGHBOUR_COUNT, graph::Node, storage::InMemStorage};

use super::{graph::Graph, plotter::Plotter, vector::generate_random_vectors};

pub fn debug(seed_dataset_size: usize, vector_value_range: std::ops::Range<f32>) {
    let test_vectors = generate_random_vectors(seed_dataset_size, &vector_value_range, 2);

    // let disk = crate::storage::NaiveDisk::new(
    //     VECTOR_DIMENSION,
    //     MAX_NEIGHBOUR_COUNT,
    //     "disk.index",
    //     "disk.free",
    // )
    // .unwrap();

    let in_mem = InMemStorage::new();
    let mut graph = Graph::new(&test_vectors, 2, MAX_NEIGHBOUR_COUNT, Box::new(in_mem)).unwrap();
    let mut plotter = Plotter::new(vector_value_range.clone());

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

    let inserted_node = graph.insert(vec![1000.0, 1000.0], 0, 1.2, 10).unwrap();
    plotter.set_connected_nodes(&graph.storage.get_all_nodes());
    plotter.set_isolated_nodes(&vec![inserted_node]);
    plotter
        .plot(&format!("{}/graph-3.png", path), "inserted")
        .unwrap();

    // disk::write_to_disk(&graph);
}
