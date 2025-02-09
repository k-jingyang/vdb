use std::sync::Mutex;

use crate::Node;

use super::GraphStorage;

// FreshDisk is the storage implementation of the system described in the FreshDiskANN paper
//
pub struct FreshDisk {
    long_term_index: crate::NaiveDisk,
    delete_list: Vec<u32>, // delete not implemented yet
    ro_temp_indices: Mutex<Vec<Node>>,
    rw_temp_index: Mutex<Vec<Node>>,
}

impl FreshDisk {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // start async thread to periodically
    }
}

impl GraphStorage for FreshDisk {
    fn add_nodes(&mut self, data: &[Vec<f32>]) -> std::io::Result<Vec<u32>> {
        todo!()
    }

    fn get_node(&self, node_id: u32) -> std::io::Result<Node> {
        todo!()
    }

    fn set_connections(
        &mut self,
        node_index: u32,
        connections: &std::collections::HashSet<u32>,
    ) -> std::io::Result<()> {
        todo!()
    }

    fn get_random_node(&self) -> Option<Node> {
        todo!()
    }

    fn get_all_node_indexes(&self) -> Vec<u32> {
        todo!()
    }

    fn get_all_nodes(&self) -> Vec<Node> {
        todo!()
    }
}
