use std::fs;

use chrono::Local;

use crate::{
    constant::MAX_NEIGHBOUR_COUNT, constant::VECTOR_DIMENSION, graph::Node, storage::InMemStorage,
};

use super::{graph::Graph, plotter::Plotter, vector::generate_random_vectors};

pub fn debug(
    seed_dataset_size: usize,
    vector_value_range: std::ops::Range<f32>,
    storage: Box<dyn crate::storage::GraphStorage>,
) {
    let test_vectors = generate_random_vectors(seed_dataset_size, &vector_value_range, 2);
    let mut graph = Graph::new(&test_vectors, 2, MAX_NEIGHBOUR_COUNT, storage).unwrap();
    let mut plotter = Plotter::new(vector_value_range.clone());

    // plot initial
    let nodes = graph.storage.get_all_nodes();
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
    plotter.set_connected_nodes(&graph.storage.get_all_nodes());
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
    plotter.set_connected_nodes(&graph.storage.get_all_nodes());
    plotter
        .plot(&format!("{}/graph-2.png", path), "second pass, α=1.2")
        .unwrap();

    // insert new node
    let inserted_node = graph.insert(vec![1000.0, 1000.0], 1, 1.2, 10).unwrap();
    plotter.set_connected_nodes(&graph.storage.get_all_nodes());
    plotter.set_isolated_nodes(&vec![inserted_node]);
    plotter
        .plot(&format!("{}/graph-3.png", path), "inserted")
        .unwrap();
}
