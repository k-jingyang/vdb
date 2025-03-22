use crate::{prelude::Error, prelude::*, Node};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, RwLock},
    time::Duration,
};

use super::GraphStorage;

// FreshDisk is the storage implementation of the system described in the FreshDiskANN paper
// Note: current impl is buggy because of data races
pub struct FreshDisk {
    long_term_index: Arc<RwLock<crate::NaiveDisk>>,
    delete_list: Vec<u32>, // delete not implemented yet
    ro_temp_index: Arc<RwLock<VecDeque<HashMap<u32, Node>>>>,
    rw_temp_index: Arc<RwLock<HashMap<u32, Node>>>,
    next_node_index: u32,
}

impl FreshDisk {
    pub fn new(
        dimensions: u16,
        max_neighbor_count: u8,
        index_path: &str,
        free_path: &str,
    ) -> Result<Self> {
        let long_term_index = Arc::new(RwLock::new(crate::NaiveDisk::new(
            dimensions,
            max_neighbor_count,
            index_path,
            free_path,
        )?));

        let ro_temp_index = Arc::new(RwLock::new(VecDeque::new()));
        let rw_temp_index = RwLock::new(HashMap::new());

        let fresh_disk = FreshDisk {
            long_term_index: long_term_index.clone(),
            delete_list: Vec::new(),
            ro_temp_index: ro_temp_index.clone(),
            rw_temp_index: Arc::new(rw_temp_index),
            next_node_index: 1, // node_index=0 is reserved to indicate that node doesn't exist
        };

        std::thread::spawn(move || {
            Self::periodic_flush(long_term_index, ro_temp_index);
        });

        Ok(fresh_disk)
    }

    fn check_and_convert_rw_index(&mut self) {
        let rw_temp = self.rw_temp_index.read().unwrap();
        if rw_temp.len() < 10000 {
            return;
        }
        drop(rw_temp);

        let old_rw_temp_index = std::mem::replace(
            &mut self.rw_temp_index,
            Arc::new(RwLock::new(HashMap::new())),
        );
        let mut ro_temp = self.ro_temp_index.write().unwrap();
        ro_temp.push_back(old_rw_temp_index.write().unwrap().clone());
    }

    // TODO: is there a way to compact the ro index
    fn periodic_flush(
        long_term_index: Arc<RwLock<crate::NaiveDisk>>,
        ro_temp_index: Arc<RwLock<VecDeque<HashMap<u32, Node>>>>,
    ) {
        loop {
            // TODO: What's a good configuration for this flushing
            std::thread::sleep(Duration::from_millis(300));

            // To prevent a long lock on ro_temp_index, we get what we need to flush,
            // flush it, then remove it from the ro_temp_index
            //
            //
            // ro_temp_index lock here blocks check_and_convert_rw_index
            let ro_temp = ro_temp_index.read().unwrap();
            if ro_temp.len() == 0 {
                continue;
            }
            println!("size of ro_temp: {}", ro_temp.len());
            println!("size of to_flush: {}", ro_temp.front().unwrap().len());
            let to_flush = ro_temp.front().unwrap().clone();
            drop(ro_temp);

            let mut long_term = long_term_index.write().unwrap();
            println!("Flushing from ro_temp to long_term...");

            // Flush the ro_temp_index from the back
            // TODO: this flush can be improved using io_uring
            for (_, node) in to_flush.iter() {
                long_term.set_node(node).unwrap();
            }
            drop(long_term);

            ro_temp_index.write().unwrap().pop_front();
            println!("Flushing done...");
        }
    }
}

impl GraphStorage for FreshDisk {
    fn add_nodes(&mut self, data: &[Vec<f32>]) -> Result<Vec<u32>> {
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
            self.check_and_convert_rw_index();
        }
        Ok(created_node_indices)
    }

    fn get_node(&self, node_id: u32) -> Result<Node> {
        if node_id == 0 {
            return Err(Error::InvalidInput("node_id=0 is reserved".to_owned()));
        }

        let from_rw_index = self.rw_temp_index.read().unwrap().get(&node_id).cloned();
        if let Some(node) = from_rw_index {
            return Ok(node);
        }

        let from_ro_index = self.ro_temp_index.read().unwrap();
        for index in from_ro_index.iter().rev() {
            if let Some(node) = index.get(&node_id) {
                return Ok(node.clone());
            }
        }

        let node = self.long_term_index.read().unwrap().get_node(node_id)?;
        Ok(node)
    }

    fn set_connections(&mut self, node_index: u32, connections: &HashSet<u32>) -> Result<()> {
        if node_index == 0 {
            return Err(Error::InvalidInput("node_id=0 is reserved".to_owned()));
        }

        let mut node = self.get_node(node_index)?;
        node.connected = connections.clone();
        self.rw_temp_index.write().unwrap().insert(node_index, node);
        self.check_and_convert_rw_index();
        Ok(())
    }

    fn get_random_node(&self) -> Option<Node> {
        self.get_node(1).ok()
    }

    fn get_all_node_indexes(&self) -> Result<Vec<u32>> {
        let long_term_index = self.long_term_index.read().unwrap();
        let ro_index = self.ro_temp_index.read().unwrap();
        let rw_index = self.rw_temp_index.read().unwrap();

        let mut node_indexes: HashSet<u32> = long_term_index
            .get_all_node_indexes()?
            .into_iter()
            .collect();

        ro_index.iter().for_each(|index| {
            index.keys().for_each(|node_id| {
                node_indexes.insert(*node_id);
            });
        });

        rw_index.keys().for_each(|node_id| {
            node_indexes.insert(*node_id);
        });

        let mut node_indexes: Vec<u32> = node_indexes.into_iter().collect();
        node_indexes.sort_unstable();
        Ok(node_indexes)
    }

    fn get_all_nodes(&self) -> Result<HashMap<u32, Node>> {
        let long_term_index = self.long_term_index.read().unwrap();
        let ro_index = self.ro_temp_index.read().unwrap();
        let rw_index = self.rw_temp_index.read().unwrap();

        // order of insertion must be long term index > ro index > rw index
        let mut all_nodes: HashMap<u32, Node> = long_term_index.get_all_nodes()?;

        // must read from the first ro_index, because outdated data will be before updated data
        ro_index.iter().for_each(|index| {
            for (node_id, node) in index.iter() {
                all_nodes.insert(*node_id, node.clone());
            }
        });

        rw_index.iter().for_each(|(node_id, node)| {
            all_nodes.insert(*node_id, node.clone());
        });

        Ok(all_nodes)
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

        assert_eq!(ids, vec![1, 2]);
        fresh_disk
            .set_connections(1, &HashSet::from([2u32]))
            .unwrap();
        fresh_disk
            .set_connections(2, &HashSet::from([1u32]))
            .unwrap();

        // Retrieve nodes and verify
        let retrieved_node1 = fresh_disk.get_node(1).unwrap();
        let retrieved_node2 = fresh_disk.get_node(2).unwrap();

        assert_eq!(1, retrieved_node1.id);
        assert_eq!(vec![1.0, 2.0], retrieved_node1.vector);
        assert_eq!(HashSet::from([2]), retrieved_node1.connected);

        assert_eq!(2, retrieved_node2.id);
        assert_eq!(vec![3.0, 4.0], retrieved_node2.vector);
        assert_eq!(HashSet::from([1]), retrieved_node2.connected);
    }
}
