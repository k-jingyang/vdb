use std::{
    collections::{BinaryHeap, HashSet},
    fmt::Binary,
    hash::Hash,
};

use std::cmp::Reverse;

use plotters::{
    chart::ChartBuilder,
    prelude::{BitMapBackend, IntoDrawingArea},
    series::LineSeries,
    style::{RED, WHITE},
};
use rand::{thread_rng, Rng};

// Define a constant for the array size
const VECTOR_DIMENSION: usize = 2;
const GRAPH_RANGE: (std::ops::Range<f64>, std::ops::Range<f64>) = (0.0..2000.0, 0.0..2000.0);
const VECTOR_VALUE_RANGE: (std::ops::Range<f64>) = (0.0..2000.0);
const SEED_DATASET_SIZE: u32 = 500;

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

    fn plot(&self, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = file_name;
        let root = BitMapBackend::new(output_path, (1024, 1024)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        // Define the chart and the axes
        let mut chart = ChartBuilder::on(&root)
            .caption("Vectors", ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(GRAPH_RANGE.0, GRAPH_RANGE.1)?; // Adjust the ranges as needed

        chart.configure_mesh().draw()?;

        // Define which points to connect
        for i in 0..self.nodes.len() {
            let point_1 = self.nodes[i].vector;
            for connected_node in self.nodes[i].connected.iter() {
                let point_2 = self.nodes[*connected_node].vector;
                let connected_points = vec![(point_1[0], point_1[1]), (point_2[0], point_2[1])];
                chart.draw_series(LineSeries::new(connected_points, RED))?;
            }
        }

        root.present()?;

        // println!("{:?}", self.nodes);
        Ok(())
    }

    fn greedy_search(
        &self,
        start_node_index: usize,
        query_node: [f64; VECTOR_DIMENSION],
        k: usize,
        search_list_size: usize,
    ) -> (Vec<usize>, HashSet<usize>) {
        let mut closests: BinaryHeap<(i64, usize)> = BinaryHeap::new();
        let mut visited: HashSet<usize> = HashSet::new();

        // .0 is the distance from query_node_index, .1 is the index of the node
        let mut to_visit: BinaryHeap<Reverse<(i64, usize)>> = BinaryHeap::new();

        let start_node_distance = distance(query_node, self.nodes[start_node_index].vector);
        to_visit.push(Reverse((start_node_distance, start_node_index)));
        closests.push((start_node_distance, start_node_index));

        while to_visit.len() > 0 {
            // TODO: there's a better way at setting visiting
            let visiting = to_visit.pop().unwrap().0 .1;
            visited.insert(visiting);
            for neighbor in &self.nodes[visiting].connected {
                if visited.contains(neighbor) {
                    continue;
                }

                let distance_to_q = distance(self.nodes[*neighbor].vector, query_node);
                to_visit.push(Reverse((distance_to_q, *neighbor)));
                closests.push((distance_to_q, *neighbor));
            }

            while closests.len() > search_list_size {
                closests.pop();
            }

            to_visit.clear();
            closests.iter().for_each(|node| {
                if !visited.contains(&node.1) {
                    to_visit.push(Reverse(*node));
                }
            })
        }

        let k_closests: Vec<usize> = closests.into_sorted_vec()[0..k]
            .iter()
            .map(|x| x.1)
            .collect();

        return (k_closests, visited);
    }
}

#[derive(Debug)]
struct Node {
    vector: [f64; VECTOR_DIMENSION],
    connected: HashSet<usize>,
}

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

    // find medoid
    graph.plot("graph-start.png").unwrap();
    let start_node_index = thread_rng().gen_range(0..graph.nodes.len());
    graph.greedy_search(start_node_index, [0.0, 0.0], 3, 10);
}

fn index(input: &Vec<[f64; VECTOR_DIMENSION]>, r: usize) {}

fn distance(a: [f64; VECTOR_DIMENSION], b: [f64; VECTOR_DIMENSION]) -> i64 {
    let mut sum: f64 = 0.0;
    for i in 0..VECTOR_DIMENSION {
        sum += (a[i] - b[i]).powi(2);
    }
    sum.sqrt().round() as i64
}
