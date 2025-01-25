use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::{
    constant::{SEED_DATASET_SIZE, VECTOR_DIMENSION, VECTOR_VALUE_RANGE},
    graph::{Graph, Node},
    plotter,
};

pub(super) fn init() {
    let mut test_vectors = Vec::new();
    for _ in 0..SEED_DATASET_SIZE {
        let mut arr = [0f64; VECTOR_DIMENSION];
        for i in 0..VECTOR_DIMENSION {
            let val = thread_rng().gen_range(VECTOR_VALUE_RANGE);
            arr[i] = val
        }
        test_vectors.push(arr);
    }

    let graph = Graph::new(&test_vectors, 5);

    let start_node_index = thread_rng().gen_range(0..graph.nodes.len());
    let (closests, visited) = graph.greedy_search(start_node_index, [1800.0, 0.0], 3, 10);

    let mut plotter = plotter::Plotter::new();

    plotter.add_all_nodes(&graph.nodes);

    // let closests_nodes = closests
    //     .iter()
    //     .map(|i| graph.nodes[*i].clone())
    //     .collect::<Vec<Node>>();
    // plotter.color_specific_nodes(&closests_nodes);

    plotter.plot("graph-start.png").unwrap();
}

fn index(database: &mut Graph) {
    let range: Vec<usize> = (0..database.nodes.len()).collect();
    let mut rng = thread_rng();
    range.shuffle(rng);

    for i in range {
        let (closests, visited) = database.greedy_search(i, [1800.0, 0.0], 3, 10);
    }
}
