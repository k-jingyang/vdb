use crate::prelude::*;
use crate::storage::GraphStorage;
use rand::{seq::SliceRandom, thread_rng, Rng};
use simsimd::SpatialSimilarity;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
};

pub struct Graph {
    pub(crate) storage: Box<dyn GraphStorage>,
    pub(crate) max_neighbour_count: usize,
}

impl Graph {
    pub fn new(
        input: &[Vec<f32>],
        r: usize,
        max_neighbour_count: u8,
        mut store: Box<dyn GraphStorage>,
    ) -> Result<Self> {
        let new_nodes_indices = store.add_nodes(input)?;
        let mut new_nodes: Vec<Node> = Vec::new();

        for i in 0..new_nodes_indices.len() {
            let node_index = new_nodes_indices[i];
            let node = Node {
                id: node_index,
                vector: input[i].clone(),
                connected: HashSet::new(),
            };
            new_nodes.push(node);
        }

        // for each node, connect it to random r nodes
        for i in 0..new_nodes.len() {
            for _ in 0..r {
                if new_nodes[i].connected.len() >= max_neighbour_count as usize {
                    continue;
                }
                loop {
                    let random_index = thread_rng().gen_range(0..new_nodes.len());
                    if random_index != i
                        && new_nodes[random_index].connected.len() < max_neighbour_count as usize
                    {
                        let random_node_id = new_nodes[random_index].id.clone();
                        new_nodes[i].connected.insert(random_node_id);
                        let i_node_id = new_nodes[i].id.clone();
                        new_nodes[random_index].connected.insert(i_node_id);
                        break;
                    }
                }
            }
        }

        new_nodes.iter().for_each(|n| {
            store.set_connections(n.id, &n.connected).unwrap();
        });

        Ok(Graph {
            storage: store,
            max_neighbour_count: max_neighbour_count as usize,
        })
    }

    pub fn greedy_search_random_start(
        &self,
        query_node: &[f32],
        k: usize,
        search_list_size: usize,
    ) -> (Vec<u32>, HashSet<u32>) {
        let all_node_indexes = self.storage.get_all_node_indexes();
        let random_index = *all_node_indexes.unwrap().choose(&mut thread_rng()).unwrap();
        self.greedy_search(random_index, query_node, k, search_list_size)
    }

    pub fn greedy_search(
        &self,
        start_node_index: u32,
        query_node: &[f32],
        k: usize,
        search_list_size: usize,
    ) -> (Vec<u32>, HashSet<u32>) {
        let mut closest_l: BinaryHeap<(i64, u32)> = BinaryHeap::new();
        let mut visited: HashSet<u32> = HashSet::new();
        // .0 is the distance from query_node_index, .1 is the index of the node
        let mut to_visit: BinaryHeap<Reverse<(i64, u32)>> = BinaryHeap::new();

        let start_node = self.storage.get_node(start_node_index).unwrap();
        // Initial distance
        let start_node_distance = euclidean_distance(query_node, &start_node.vector);
        to_visit.push(Reverse((start_node_distance, start_node_index)));
        closest_l.push((start_node_distance, start_node_index));

        while let Some(Reverse((_, visiting))) = to_visit.pop() {
            visited.insert(visiting);

            let visiting_node = self.storage.get_node(visiting).unwrap();
            for neighbor in &visiting_node.connected {
                if visited.contains(neighbor) {
                    continue;
                }

                let visiting_node_neighbor = self.storage.get_node(*neighbor).unwrap();
                let distance_to_q = euclidean_distance(&visiting_node_neighbor.vector, query_node);

                closest_l.push((distance_to_q, *neighbor));
            }

            // since closest_k is a max heap, we will keep the k closest after popping
            while closest_l.len() > search_list_size {
                closest_l.pop();
            }

            to_visit.clear();
            closest_l.iter().for_each(|node| {
                if !visited.contains(&node.1) {
                    to_visit.push(Reverse(*node));
                }
            })
        }

        let k_closests: Vec<u32> = closest_l
            .into_sorted_vec()
            .iter()
            .take(k)
            .map(|x| x.1)
            .collect();

        return (k_closests, visited);
    }

    pub(super) fn robust_prune(
        &mut self,
        p_index: u32,
        visited: &HashSet<u32>,
        distance_threshold: f32,
        degree_bound: usize,
    ) -> Result<Node> {
        // add all nodes that was visited to try to reach p (excluding p) into working set
        let mut working_set = visited.clone();
        working_set.retain(|x| *x != p_index);

        // add all nodes connected to p into working set
        let p_node = self.storage.get_node(p_index).unwrap();

        working_set.extend(p_node.connected.iter());

        let mut distance_heap: BinaryHeap<Reverse<(i64, u32)>> = BinaryHeap::new();
        for node_index in working_set.iter() {
            let working_set_node = self.storage.get_node(*node_index).unwrap();
            let distance_from_p = euclidean_distance(&p_node.vector, &working_set_node.vector);
            distance_heap.push(Reverse((distance_from_p, *node_index)));
        }

        // reset p's connected
        let mut p_node_connections: HashSet<u32> = HashSet::new();

        while let Some(Reverse((_, min_node_index))) = distance_heap.pop() {
            // add min_node to p_index's connected
            // note: the reverse connection is added by the caller of this method
            p_node_connections.insert(min_node_index);
            if p_node_connections.len() == degree_bound {
                break;
            }

            let min_node = self.storage.get_node(min_node_index).unwrap();

            distance_heap.retain(|x| {
                let comparison_node = self.storage.get_node(x.0 .1).unwrap();

                let distance_to_min_node =
                    euclidean_distance(&min_node.vector, &comparison_node.vector) as f64;
                let distance_to_p =
                    euclidean_distance(&comparison_node.vector, &p_node.vector) as f64;
                distance_to_min_node * distance_threshold as f64 > distance_to_p
            });
        }

        self.storage.set_connections(p_index, &p_node_connections)?;

        Ok(Node {
            id: p_node.id,
            vector: p_node.vector,
            connected: p_node_connections,
        })
    }

    pub fn index(&mut self, distance_threshold: f32) -> Result<()> {
        let start_node = self.storage.get_random_node().unwrap();
        let start_node_index = start_node.id;

        let mut node_indices: Vec<u32> = self.storage.get_all_node_indexes()?;
        let mut rng = thread_rng();
        node_indices.shuffle(&mut rng);

        for node_index in node_indices {
            let query_node = self.storage.get_node(node_index)?;
            let (_, visited) = self.greedy_search(start_node_index, &query_node.vector, 3, 10);

            let query_node = self.robust_prune(
                node_index,
                &visited,
                distance_threshold,
                self.max_neighbour_count,
            )?;

            let connected_node_indices = query_node.connected.clone();
            for connected_node_index in connected_node_indices.iter() {
                let mut connected_node = self.storage.get_node(*connected_node_index).unwrap();
                connected_node.connected.insert(node_index);

                if connected_node.connected.len() > self.max_neighbour_count {
                    self.robust_prune(
                        *connected_node_index,
                        &connected_node.connected,
                        distance_threshold,
                        self.max_neighbour_count,
                    )?;
                } else {
                    self.storage
                        .set_connections(*connected_node_index, &connected_node.connected)?;
                }
            }
        }

        Ok(())
    }

    pub fn insert(
        &mut self,
        insert_vector: Vec<f32>,
        start_node_index: u32,
        distance_threshold: f32,
        search_list_size: usize,
    ) -> Result<Node> {
        let (_, visited) =
            self.greedy_search(start_node_index, &insert_vector, 1, search_list_size);
        let new_node_index = self.storage.add_nodes(&[insert_vector])?[0];

        let new_node = self.robust_prune(
            new_node_index,
            &visited,
            distance_threshold,
            self.max_neighbour_count,
        )?;

        let connected_node_indices = new_node.connected.clone();
        for connected_node_index in connected_node_indices.iter() {
            let mut connected_node = self.storage.get_node(*connected_node_index).unwrap();
            connected_node.connected.insert(new_node_index);

            if connected_node.connected.len() > self.max_neighbour_count {
                self.robust_prune(
                    *connected_node_index,
                    &connected_node.connected,
                    distance_threshold,
                    self.max_neighbour_count,
                )?;
            } else {
                self.storage
                    .set_connections(*connected_node_index, &connected_node.connected)?;
            }
        }

        Ok(new_node)
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) id: u32,
    pub(crate) vector: Vec<f32>,
    pub(crate) connected: HashSet<u32>,
}

fn euclidean_distance(a: &[f32], b: &[f32]) -> i64 {
    let l2sq_dist = f32::l2sq(a, b);
    return l2sq_dist.unwrap() as i64;
}
