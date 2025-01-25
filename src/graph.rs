use crate::constant::VECTOR_DIMENSION;
use rand::{thread_rng, Rng};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
};

pub(super) struct Graph {
    pub(super) nodes: Vec<Node>,
}

impl Graph {
    pub(super) fn new(input: &Vec<[f64; VECTOR_DIMENSION]>, r: usize) -> Self {
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

    /// Performs a greedy search starting from a given node index to find the k closest nodes
    /// to a query node based on their vector distances.
    ///
    /// # Arguments
    ///
    /// * `start_node_index` - The index of the node to start the search from.
    /// * `query_node` - The vector representing the query node.
    /// * `k` - The number of closest nodes to find.
    /// * `search_list_size` - The maximum size of the search list maintained during the search.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A vector of indices of the k closest nodes to the query node.
    /// * A set of indices of nodes that were visited during the search.
    pub(super) fn greedy_search(
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

    pub(super) fn robust_prune(
        &mut self,
        node_index: usize,
        visited: &HashSet<usize>,
        distance_threshold: i64,
        degree_bound: usize,
    ) {
        let mut visited_nodes = visited.clone();
        // add all nodes connected to node_index into V
        for i in self.nodes[node_index].connected.iter() {
            visited_nodes.insert(*i);
        }
        // remove node_index from V
        visited_nodes.remove(&node_index);

        // set node_index's connected to empty
        self.nodes[node_index].connected.clear();

        let distance_heap: BinaryHeap<Reverse<(i64, usize)>> = BinaryHeap::new();

        while visited_nodes.len() > 0 {
            // for all visited nodes, get the node i with min distance to node_index
            // add i to nodex_index's connected, if N out == degree_bound; stop
            //
            if self.nodes[node_index].connected.len() == degree_bound {
                break;
            }

            // for node p' in visited_nodes, if threshold * d(i, p') <= d(p, p'), remove p' from V
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Node {
    pub(super) vector: [f64; VECTOR_DIMENSION],
    pub(super) connected: HashSet<usize>,
}
fn distance(a: [f64; VECTOR_DIMENSION], b: [f64; VECTOR_DIMENSION]) -> i64 {
    let mut sum: f64 = 0.0;
    for i in 0..VECTOR_DIMENSION {
        sum += (a[i] - b[i]).powi(2);
    }
    sum.sqrt().round() as i64
}
