use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, RwLock},
};

use crate::Node;

use super::GraphStorage;

// FreshDisk is the storage implementation of the system described in the FreshDiskANN paper
// Note: current impl is buggy because of race conditions
pub struct FreshDisk {
    long_term_index: RwLock<crate::NaiveDisk>,
    delete_list: Vec<u32>, // delete not implemented yet
    ro_temp_index: RwLock<HashMap<u32, Node>>,
    rw_temp_index: RwLock<HashMap<u32, Node>>,
    next_node_index: u32,
}

impl FreshDisk {
    pub fn new(
        dimensions: u16,
        max_neighbor_count: u8,
        index_path: &str,
        free_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let fresh_disk = FreshDisk {
            long_term_index: RwLock::new(crate::NaiveDisk::new(
                dimensions,
                max_neighbor_count,
                index_path,
                free_path,
            )?),
            delete_list: Vec::new(),
            ro_temp_index: RwLock::new(HashMap::new()),
            rw_temp_index: RwLock::new(HashMap::new()),
            next_node_index: 0,
        };

        // TODO: start async thread to periodically flush ro_temp_index to long_term_index

        Ok(fresh_disk)
    }

    fn check_and_convert_rw_index(&mut self) {
        // TODO: if rw_index reach max limit, flush rw_index to ro_index
    }
}

impl GraphStorage for FreshDisk {
    fn add_nodes(&mut self, data: &[Vec<f32>]) -> std::io::Result<Vec<u32>> {
        let mut created_node_indices = Vec::new();

        for datum in data {
            let node = Node {
                id: self.next_node_index,
                vector: datum.clone(),
                connected: HashSet::new(),
            };
            self.rw_temp_index.write().unwrap().insert(node.id, node);
            created_node_indices.push(self.next_node_index);
            self.next_node_index += 1;
        }
        Ok(created_node_indices)
    }

    fn get_node(&self, node_id: u32) -> std::io::Result<Node> {
        let from_rw_index = self.rw_temp_index.read().unwrap().get(&node_id).cloned();
        if let Some(node) = from_rw_index {
            return Ok(node);
        }

        let from_ro_index = self.ro_temp_index.read().unwrap().get(&node_id).cloned();
        if let Some(node) = from_ro_index {
            return Ok(node);
        }

        let node = self.long_term_index.read().unwrap().get_node(node_id)?;
        Ok(node)
    }

    fn set_connections(
        &mut self,
        node_index: u32,
        connections: &std::collections::HashSet<u32>,
    ) -> std::io::Result<()> {
        let mut node = self.get_node(node_index)?;
        node.connected = connections.clone();
        self.rw_temp_index.write().unwrap().insert(node_index, node);
        Ok(())
    }

    fn get_random_node(&self) -> Option<Node> {
        self.get_node(0).ok()
    }

    // unsorted
    fn get_all_node_indexes(&self) -> Vec<u32> {
        let long_term_index = self.long_term_index.read().unwrap();
        let ro_index = self.ro_temp_index.read().unwrap();
        let rw_index = self.rw_temp_index.read().unwrap();

        let mut node_indexes: HashSet<u32> =
            long_term_index.get_all_node_indexes().into_iter().collect();

        ro_index.keys().for_each(|node_id| {
            node_indexes.insert(*node_id);
        });

        rw_index.keys().for_each(|node_id| {
            node_indexes.insert(*node_id);
        });

        node_indexes.into_iter().collect()
    }

    // unsorted
    fn get_all_nodes(&self) -> Vec<Node> {
        let long_term_index = self.long_term_index.read().unwrap();
        let ro_index = self.ro_temp_index.read().unwrap();
        let rw_index = self.rw_temp_index.read().unwrap();

        let mut all_nodes: HashMap<u32, Node> = long_term_index
            .get_all_nodes()
            .into_iter()
            .map(|node| (node.id, node))
            .collect();

        ro_index.iter().for_each(|(node_id, node)| {
            all_nodes.insert(*node_id, node.clone());
        });

        rw_index.iter().for_each(|(node_id, node)| {
            all_nodes.insert(*node_id, node.clone());
        });

        all_nodes.into_iter().map(|(_, node)| node).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_add_nodes_and_get_node() {
        let temp_dir = env::temp_dir();
        let index_path = temp_dir.as_path().join("test.index");
        let free_path = temp_dir.as_path().join("test.free");

        // Create a DiskStorage instance
        let mut fresh_disk = FreshDisk::new(
            2,
            3,
            index_path.to_str().unwrap(),
            free_path.to_str().unwrap(),
        )
        .unwrap();

        // Add nodes to the storage
        let ids = fresh_disk
            .add_nodes(&[vec![1.0, 2.0], vec![3.0, 4.0]])
            .unwrap();

        assert_eq!(ids, vec![0, 1]);
        fresh_disk
            .set_connections(0, &HashSet::from([1u32]))
            .unwrap();
        fresh_disk
            .set_connections(1, &HashSet::from([0u32]))
            .unwrap();

        // Retrieve nodes and verify
        let retrieved_node1 = fresh_disk.get_node(0).unwrap();
        let retrieved_node2 = fresh_disk.get_node(1).unwrap();

        assert_eq!(0, retrieved_node1.id);
        assert_eq!(vec![1.0, 2.0], retrieved_node1.vector);
        assert_eq!(HashSet::from([1]), retrieved_node1.connected);

        assert_eq!(1, retrieved_node2.id);
        assert_eq!(vec![3.0, 4.0], retrieved_node2.vector);
        assert_eq!(HashSet::from([0]), retrieved_node2.connected);
    }
}
