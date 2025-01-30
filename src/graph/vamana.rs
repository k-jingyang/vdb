use rand::{thread_rng, Rng};

use crate::{
    constant::{MAX_NEIGHBOUR_COUNT, SEED_DATASET_SIZE, VECTOR_DIMENSION, VECTOR_VALUE_RANGE},
    graph::Node,
    storage::InMemStorage,
};

use super::{graph::Graph, plotter::Plotter};
use crate::storage::NaiveDisk;

pub(crate) fn init() {
    let mut test_vectors: Vec<Vec<f32>> = Vec::new();
    for _ in 0..SEED_DATASET_SIZE {
        let mut arr = [0f32; VECTOR_DIMENSION as usize];
        for i in 0..VECTOR_DIMENSION {
            let val = thread_rng().gen_range(VECTOR_VALUE_RANGE);
            arr[i as usize] = val
        }
        test_vectors.push(arr.to_vec());
    }

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

    plotter
        .plot("static/graph-initial.png", "Initial graph")
        .unwrap();

    // alpha=1
    graph.index(1.0, MAX_NEIGHBOUR_COUNT as usize).unwrap();
    let (closests, _) = graph.greedy_search(0, &[1000.0f32, 1000.0f32], 3, 10);
    let closest_nodes: Vec<Node> = closests
        .iter()
        .filter_map(|&id| graph.storage.get_node(id).ok())
        .collect();
    plotter.set_isolated_nodes(&closest_nodes);
    plotter.set_connected_nodes(&graph.storage.get_all_nodes());
    plotter
        .plot("static/graph-1.png", "first pass, α=1")
        .unwrap();

    // alpha=1.2
    graph.index(1.2, MAX_NEIGHBOUR_COUNT as usize).unwrap();
    let (closests, _) = graph.greedy_search(0, &[1000.0f32, 1000.0f32], 3, 10);
    let closest_nodes: Vec<Node> = closests
        .iter()
        .filter_map(|&id| graph.storage.get_node(id).ok())
        .collect();
    plotter.set_isolated_nodes(&closest_nodes);
    plotter.set_connected_nodes(&graph.storage.get_all_nodes());
    plotter
        .plot("static/graph-2.png", "second pass, α=1.2")
        .unwrap();
    // disk::write_to_disk(&graph);
}
