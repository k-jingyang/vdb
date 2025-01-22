use std::collections::HashSet;

use rand::{thread_rng, Rng};

// Define a constant for the array size
const VECTOR_DIMENSION: usize = 20;

struct Graph {
    nodes: Vec<Node>,
}

impl Graph {
    fn new(input: &Vec<[f64; VECTOR_DIMENSION]>, r: usize) -> Self {
        let mut nodes = input
            .iter()
            .map(|vector| Node {
                vector: vector.clone(),
                connected: HashSet::new(),
            })
            .collect::<Vec<Node>>();

        let total_size = nodes.len();

        // for each node, connect it to random r nodes
        for i in 0..nodes.len() {
            for _ in 0..r {
                if nodes[i].connected.len() >= r {
                    continue;
                }
                loop {
                    let random_index = thread_rng().gen_range(0..total_size);
                    if random_index != i {
                        nodes[i].connected.insert(random_index);
                        nodes[random_index].connected.insert(i);
                        break;
                    }
                }
            }
        }

        Graph { nodes: nodes }
    }

    fn plot(&self) {
        println!("{:?}", self.nodes);
    }
}

#[derive(Debug)]
struct Node {
    vector: [f64; VECTOR_DIMENSION],
    connected: HashSet<usize>,
}

pub(crate) fn init() {
    let seed_size = 100;
    // TODO: not sure of the range of this float array
    let mut test_vectors = Vec::new();
    for _ in 0..seed_size {
        let mut arr = [0f64; VECTOR_DIMENSION];
        thread_rng().fill(&mut arr);
        test_vectors.push(arr);
    }

    let graph = Graph::new(&test_vectors, 2);
    graph.plot();
}

// TODO: what is pub(in) vs pub(crate) vs pub(self) vs pub(super)
fn index(input: &Vec<[f64; VECTOR_DIMENSION]>, r: usize) {}

// find some way to plot
