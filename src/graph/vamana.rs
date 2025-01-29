use rand::{thread_rng, Rng};

use crate::constant::{SEED_DATASET_SIZE, VECTOR_DIMENSION, VECTOR_VALUE_RANGE};

use super::{graph::Graph, plotter::Plotter};
use crate::storage::DiskStorage;

pub(crate) fn init() {
    let mut test_vectors: Vec<Vec<f32>> = Vec::new();
    for _ in 0..SEED_DATASET_SIZE {
        let mut arr = [0f32; VECTOR_DIMENSION];
        for i in 0..VECTOR_DIMENSION {
            let val = thread_rng().gen_range(VECTOR_VALUE_RANGE);
            arr[i] = val
        }
        test_vectors.push(arr.to_vec());
    }

    let converted_vectors: Vec<&[f32]> = test_vectors.iter().map(|vec| vec.as_slice()).collect();
    let mut graph = Graph::new(converted_vectors, 5, Box::new(DiskStorage {}));
    let mut plotter = Plotter::new();

    plotter.set_connected_nodes(&graph.nodes);

    plotter
        .plot("static/graph-initial.png", "Initial graph")
        .unwrap();

    graph.index(1, 5);
    plotter.set_connected_nodes(&graph.nodes);
    plotter
        .plot("static/graph-1.png", "first pass, α=1")
        .unwrap();

    graph.index(2, 5);
    plotter.set_connected_nodes(&graph.nodes);
    plotter
        .plot("static/graph-2.png", "second pass, α=2")
        .unwrap();

    // disk::write_to_disk(&graph);
}
