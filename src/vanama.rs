use plotters::data;
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

    let mut graph = Graph::new(&test_vectors, 5);

    let start_node_index = thread_rng().gen_range(0..graph.nodes.len());

    let mut plotter = plotter::Plotter::new();

    plotter.set_connected_nodes(&graph.nodes);

    // let closests_nodes = closests
    //     .iter()
    //     .map(|i| graph.nodes[*i].clone())
    //     .collect::<Vec<Node>>();
    // plotter.color_specific_nodes(&closests_nodes);

    plotter.plot("graph-start.png").unwrap();

    index(&mut graph, 1);
    plotter.set_connected_nodes(&graph.nodes);
    plotter.plot("graph-1.png").unwrap();

    index(&mut graph, 2);
    plotter.set_connected_nodes(&graph.nodes);
    plotter.plot("graph-2.png").unwrap();

    index(&mut graph, 3);
    plotter.set_connected_nodes(&graph.nodes);
    plotter.plot("graph-3.png").unwrap();
}

fn index(database: &mut Graph, distance_threshold: i64) {
    const DEGREE_BOUND: usize = 5;

    let start_node_index = thread_rng().gen_range(0..database.nodes.len());

    let mut nodes: Vec<usize> = (0..database.nodes.len()).collect();
    let mut rng = thread_rng();
    nodes.shuffle(&mut rng);

    for node in nodes {
        let (_, visited) =
            database.greedy_search(start_node_index, database.nodes[node].vector, 3, 10);
        database.robust_prune(node, &visited, 1, DEGREE_BOUND);

        let connected_nodes = &database.nodes[node].connected.clone();
        for connected_node in connected_nodes.iter() {
            database.nodes[*connected_node].connected.insert(node);

            let connected_node_outgoing = database.nodes[*connected_node].connected.clone();
            if database.nodes[*connected_node].connected.len() > DEGREE_BOUND {
                database.robust_prune(
                    *connected_node,
                    &connected_node_outgoing,
                    distance_threshold,
                    DEGREE_BOUND,
                );
            }
        }
    }
}
