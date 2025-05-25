use std::fs;

use chrono::Local;
use vdb::{vector::generate_random_vectors, InMemStorage, Node};

use crate::{new_index_storage, Storage, MAX_NEIGHBOUR_COUNT};

pub(super) fn debug(
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
        .plot(&format!("{}/graph-1.png", path), "first pass, α=1.0")
        .unwrap();

    // alpha=1.0
    graph.index(1.0).unwrap();
    let (closests, _) = graph.greedy_search(1, &[1000.0f32, 1000.0f32], 3, 10);
    let closest_nodes: Vec<Node> = closests
        .iter()
        .filter_map(|&id| graph.index_store.get_node(id).ok())
        .collect();
    plotter.set_isolated_nodes(&closest_nodes);
    plotter.set_connected_nodes(&graph.index_store.get_all_nodes().unwrap());
    plotter
        .plot(&format!("{}/graph-2.png", path), "second pass, α=1.0")
        .unwrap();

    // insert new node
    let inserted_node = graph
        .insert(vec![1000.0, 1000.0], "".to_string(), 1, 1.0, 10)
        .unwrap();
    plotter.set_connected_nodes(&graph.index_store.get_all_nodes().unwrap());
    plotter.set_isolated_nodes(&vec![inserted_node]);
    plotter
        .plot(&format!("{}/graph-3.png", path), "inserted")
        .unwrap();
}
