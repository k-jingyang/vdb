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
    /// Creates a new graph from the given input vectors. The graph is a random graph
    /// where each node is connected to `r` other nodes. The graph is undirected and
    /// unweighted.
    ///
    /// # Arguments
    ///
    /// * `input` - A vector of vectors representing the vectors to create nodes from.
    /// * `r` - The number of random nodes to connect each node to.
    ///
    /// # Returns
    ///
    /// A new `Graph` with the specified properties.
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
        let mut closest_l: BinaryHeap<(i64, usize)> = BinaryHeap::new();
        let mut visited: HashSet<usize> = HashSet::new();
        // .0 is the distance from query_node_index, .1 is the index of the node
        let mut to_visit: BinaryHeap<Reverse<(i64, usize)>> = BinaryHeap::new();

        // Initial distance
        let start_node_distance =
            euclidean_distance(query_node, self.nodes[start_node_index].vector);
        to_visit.push(Reverse((start_node_distance, start_node_index)));
        closest_l.push((start_node_distance, start_node_index));

        while let Some(Reverse((_, visiting))) = to_visit.pop() {
            visited.insert(visiting);

            for neighbor in &self.nodes[visiting].connected {
                if visited.contains(neighbor) {
                    continue;
                }

                let distance_to_q = euclidean_distance(self.nodes[*neighbor].vector, query_node);

                closest_l.push((distance_to_q, *neighbor));
            }

            // since closest_k is a max heap, we will keep the k closest after popping
            while closest_l.len() > search_list_size {
                closest_l.pop();
            }

            // note: super wasteful here
            to_visit.clear();
            closest_l.iter().for_each(|node| {
                if !visited.contains(&node.1) {
                    to_visit.push(Reverse(*node));
                }
            })
        }

        let k_closests: Vec<usize> = closest_l
            .into_sorted_vec()
            .iter()
            .take(k)
            .map(|x| x.1)
            .collect();

        return (k_closests, visited);
    }

    pub(super) fn robust_prune(
        &mut self,
        p_index: usize,
        visited: &HashSet<usize>,
        distance_threshold: i64,
        degree_bound: usize,
    ) {
        // add all nodes that was visited to try to reach p (exclusing p) into working set
        let mut working_set = visited.clone();
        working_set.retain(|x| *x != p_index);

        // add all nodes connected to p into working set
        working_set.extend(self.nodes[p_index].connected.iter());

        let mut distance_heap: BinaryHeap<Reverse<(i64, usize)>> = BinaryHeap::new();
        for node_index in working_set.iter() {
            let distance_from_p =
                euclidean_distance(self.nodes[p_index].vector, self.nodes[*node_index].vector);
            distance_heap.push(Reverse((distance_from_p, *node_index)));
        }

        // reset p's connected
        self.nodes[p_index].connected.clear();

        while let Some(Reverse((_, min_node))) = distance_heap.pop() {
            // add min_node to p_index's connected
            // note: the reverse connection is added by the caller of this method
            self.nodes[p_index].connected.insert(min_node);
            if self.nodes[p_index].connected.len() == degree_bound {
                break;
            }

            let min_node_vector = self.nodes[min_node].vector;
            distance_heap.retain(|x| {
                let distance_to_min_node =
                    euclidean_distance(min_node_vector, self.nodes[x.0 .1].vector);
                let distance_to_p =
                    euclidean_distance(self.nodes[x.0 .1].vector, self.nodes[p_index].vector);
                distance_to_min_node * distance_threshold > distance_to_p
            });
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Node {
    pub(super) vector: [f64; VECTOR_DIMENSION],
    pub(super) connected: HashSet<usize>,
}
fn euclidean_distance(a: [f64; VECTOR_DIMENSION], b: [f64; VECTOR_DIMENSION]) -> i64 {
    let mut squared_distance: f64 = 0.0;
    for i in 0..VECTOR_DIMENSION {
        let difference = a[i] - b[i];
        squared_distance += difference * difference;
    }
    squared_distance.sqrt() as i64
}
