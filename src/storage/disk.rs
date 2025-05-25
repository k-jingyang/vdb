use rand::Rng;

use crate::error;
use crate::graph::Node;
use crate::prelude::Result;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Error, ErrorKind};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
};

use super::storage::IndexStore;
// disk layout
// key principle: lookup for each node index must be O(1)
//
// .free file:
// [free indices]
// [ u32 * as many]
//
// .index file:
// [metadata][nodes]
//
// where [metadata]:
// [dim][max_neighbor_count][next node index]
// [u16][      u8          ][u32            ]
//
// where [nodes]:
// [node_id][vector      ][    neighbor indexes                  ]
// [ u32   ][ f32 * dim  ][  u32 * max_neighbor_count (padded)   ]
//
// TODO:
// 1. log based input instead
// 2. decouple node_id from node index
pub struct NaiveDisk {
    dimensions: u16,
    max_neighbour_count: u8,
    next_node_index: u32,
    index_path: String,
    free_path: String,
}

impl NaiveDisk {
    // initialise a new disk backend
    // TODO: allow reading old copy
    pub fn new(
        dimensions: u16,
        max_neighbor_count: u8,
        index_path: &str,
        free_path: &str,
    ) -> Result<Self> {
        let mut index_file = BufWriter::new(File::create(index_path)?);

        // Write metadata to index file
        index_file.write_all(&dimensions.to_be_bytes())?;
        index_file.write_all(&max_neighbor_count.to_be_bytes())?;
        index_file.write_all(&(0u32).to_be_bytes())?;

        Ok(NaiveDisk {
            dimensions: dimensions,
            max_neighbour_count: max_neighbor_count,
            next_node_index: 1,
            index_path: index_path.to_string(),
            free_path: free_path.to_string(),
        })
    }

    fn index_metadata_size(&self) -> usize {
        std::mem::size_of_val(&self.dimensions)
            + std::mem::size_of_val(&self.max_neighbour_count)
            + std::mem::size_of_val(&self.next_node_index)
    }

    fn index_node_size(&self) -> usize {
        self.index_node_id_size()
            + (self.dimensions as usize * self.index_node_vector_element_size())
            + (self.max_neighbour_count as usize * self.index_node_id_size())
    }

    fn index_node_id_size(&self) -> usize {
        std::mem::size_of::<u32>()
    }

    fn index_node_vector_element_size(&self) -> usize {
        std::mem::size_of::<f32>()
    }

    fn node_offset(&self, node_index: u32) -> u64 {
        // node_index - 1 because node_index starts from 1
        self.index_metadata_size() as u64
            + ((node_index - 1) as usize * self.index_node_size()) as u64
    }

    fn node_connections_offset(&self, node_index: u32) -> u64 {
        self.node_offset(node_index)
            + self.index_node_id_size() as u64
            + (self.dimensions as usize * self.index_node_vector_element_size()) as u64
    }

    pub(crate) fn set_node(&mut self, node: &Node) -> io::Result<()> {
        if node.id == 0 {
            return Err(Error::new(ErrorKind::Other, "node id cannot be 0"));
        }
        let mut f = OpenOptions::new().write(true).open(&self.index_path)?;
        f.seek(SeekFrom::Current(self.node_offset(node.id) as i64))?;
        let mut index_file = BufWriter::new(f);

        // write node id
        index_file.write_all(&(node.id).to_be_bytes())?;

        // write vector
        for value in &node.vector {
            index_file.write_all(&value.to_be_bytes())?;
        }

        // Set neighbors
        for neighbor in &node.connected {
            index_file.write_all(&neighbor.to_be_bytes())?;
        }

        // Pad neighbor indices
        let padding = self.max_neighbour_count as usize - node.connected.len();
        for _ in 0..padding {
            index_file.write_all(&0_u32.to_be_bytes())?;
        }
        Ok(())
    }
}

impl IndexStore for NaiveDisk {
    fn set_connections(&mut self, node_index: u32, connections: &HashSet<u32>) -> Result<()> {
        if connections.len() > self.max_neighbour_count as usize {
            return Err(error::Error::InvalidInput(
                "max connections reached".to_owned(),
            ));
        }

        let mut open_index_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.index_path)?;

        let mut index_file = BufWriter::new(&mut open_index_file);

        index_file.seek(SeekFrom::Start(self.node_connections_offset(node_index)))?;

        for neighbor in connections {
            index_file.write_all(&neighbor.to_be_bytes())?;
        }

        // Pad neighbor indices
        let padding = self.max_neighbour_count as usize - connections.len();
        for _ in 0..padding {
            index_file.write_all(&0_u32.to_be_bytes())?;
        }
        Ok(())
    }

    fn add_nodes(&mut self, data: &[Vec<f32>]) -> Result<Vec<u32>> {
        let mut created_node_indices: Vec<u32> = Vec::new();

        // jump to next node offset
        let mut f = OpenOptions::new().write(true).open(&self.index_path)?;
        f.seek(SeekFrom::Current(
            self.node_offset(self.next_node_index) as i64
        ))?;

        let mut index_file = BufWriter::new(f);

        // write nodes to index file
        for datum in data {
            let node_index = self.next_node_index;

            // write node id
            index_file.write_all(&(node_index).to_be_bytes())?;
            for &value in datum {
                index_file.write_all(&value.to_be_bytes())?;
            }

            // Pad neighbor indices
            let neighbor_indices: Vec<u32> = vec![u32::MAX; self.max_neighbour_count as usize];
            for &id in &neighbor_indices {
                index_file.write_all(&id.to_be_bytes())?;
            }

            created_node_indices.push(node_index);
            self.next_node_index += 1
        }
        Ok(created_node_indices)
    }

    fn get_node(&self, node_index: u32) -> Result<Node> {
        if node_index == 0 {
            return Err(error::Error::InvalidInput("node id cannot be 0".to_owned()));
        }

        let mut index_file = File::open(&self.index_path)?;
        index_file.seek(SeekFrom::Current(self.node_offset(node_index) as i64))?;

        let mut buffer = vec![0u8; self.index_node_size()];
        index_file.read_exact(&mut buffer)?;
        let node_id = u32::from_be_bytes(buffer[0..self.index_node_id_size()].try_into().unwrap());

        // Read vector
        let mut vector: Vec<f32> = Vec::new();
        for i in 0..self.dimensions {
            let vector_offset_start =
                self.index_node_id_size() + (i as usize * self.index_node_vector_element_size());
            let vector_offset_end = vector_offset_start + self.index_node_vector_element_size();
            let vector_val = f32::from_be_bytes(
                buffer[vector_offset_start..vector_offset_end]
                    .try_into()
                    .unwrap(),
            );
            vector.push(vector_val);
        }

        // Read neighbor indices
        let mut connected: HashSet<u32> = HashSet::new();
        for i in 0..self.max_neighbour_count {
            let neighbor_offset_start = 4 + self.dimensions as usize * 4 + i as usize * 4;
            let neighbor_offset_end = neighbor_offset_start + 4; // u32 = 4 bytes
            let neighbor_index = u32::from_be_bytes(
                buffer[neighbor_offset_start..neighbor_offset_end]
                    .try_into()
                    .unwrap(),
            );

            // Ignore padding
            if neighbor_index != 0 {
                connected.insert(neighbor_index);
            }
        }

        Ok(Node {
            id: node_id,
            vector: vector,
            connected: connected,
        })
    }

    fn get_random_node(&self) -> Option<Node> {
        let mut rng = rand::thread_rng();
        let node_index: u32 = rng.gen_range(0..self.next_node_index);
        self.get_node(node_index).ok()
    }

    // scan the index file and return all node indexes
    fn get_all_node_indexes(&self) -> Result<Vec<u32>> {
        let mut node_indexes = Vec::new();

        // TODO: need to exclude free list nodes
        let mut index_file = File::open(&self.index_path)?;
        index_file.seek(SeekFrom::Current((self.index_metadata_size()) as i64))?;

        let mut buffer = vec![0u8; self.index_node_id_size()];
        loop {
            let bytes_read = index_file.read(&mut buffer)?;
            // EOF
            if bytes_read == 0 {
                break;
            }
            let node_id =
                u32::from_be_bytes(buffer[0..self.index_node_id_size()].try_into().unwrap());

            // node_id = 0 is reserved for empty
            if node_id == 0 {
                continue;
            }

            node_indexes.push(node_id);

            index_file
                .seek(SeekFrom::Current(
                    (self.index_node_size() - self.index_node_id_size()) as i64,
                ))
                .unwrap();
        }

        Ok(node_indexes)
    }

    fn get_all_nodes(&self) -> Result<HashMap<u32, Node>> {
        let mut all_nodes: HashMap<u32, Node> = HashMap::new();
        for node_index in self.get_all_node_indexes()? {
            all_nodes.insert(node_index, self.get_node(node_index)?);
        }
        Ok(all_nodes)
    }

    fn get_name(&self) -> String {
        "NaiveDisk".into()
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
        let mut disk_storage = NaiveDisk::new(
            2,
            3,
            index_path.to_str().unwrap(),
            free_path.to_str().unwrap(),
        )
        .unwrap();

        // Add nodes to the storage
        let ids = disk_storage
            .add_nodes(&[vec![1.0, 2.0], vec![3.0, 4.0]])
            .unwrap();

        assert_eq!(ids, vec![1, 2]);
        disk_storage
            .set_connections(1, &HashSet::from([2u32]))
            .unwrap();
        disk_storage
            .set_connections(2, &HashSet::from([1u32]))
            .unwrap();

        // Retrieve nodes and verify
        let retrieved_node1 = disk_storage.get_node(1).unwrap();
        let retrieved_node2 = disk_storage.get_node(2).unwrap();

        assert_eq!(1, retrieved_node1.id);
        assert_eq!(vec![1.0, 2.0], retrieved_node1.vector);
        assert_eq!(HashSet::from([2]), retrieved_node1.connected);

        assert_eq!(2, retrieved_node2.id);
        assert_eq!(vec![3.0, 4.0], retrieved_node2.vector);
        assert_eq!(HashSet::from([1]), retrieved_node2.connected);
    }

    #[test]
    fn test_set_node_and_get_node() {
        let temp_dir = env::temp_dir();
        let index_path = temp_dir.as_path().join("test_set.index");
        let free_path = temp_dir.as_path().join("test_set.free");

        // Create a DiskStorage instance
        let mut disk_storage = NaiveDisk::new(
            2,
            3,
            index_path.to_str().unwrap(),
            free_path.to_str().unwrap(),
        )
        .unwrap();

        // Add a node to the storage
        let node = Node {
            id: 5,
            vector: vec![5.0, 6.0],
            connected: HashSet::new(),
        };
        disk_storage.set_node(&node).unwrap();

        // Retrieve the node and verify
        let retrieved_node = disk_storage.get_node(5).unwrap();

        assert_eq!(5, retrieved_node.id);
        assert_eq!(vec![5.0, 6.0], retrieved_node.vector);
        assert!(retrieved_node.connected.is_empty());
    }

    #[test]
    fn test_get_all_node_indexes() {
        let temp_dir = env::temp_dir();
        let index_path = temp_dir.as_path().join("test_get_all.index");
        let free_path = temp_dir.as_path().join("test_get_all.free");

        // Create a DiskStorage instance
        let mut disk_storage = NaiveDisk::new(
            2,
            3,
            index_path.to_str().unwrap(),
            free_path.to_str().unwrap(),
        )
        .unwrap();

        // Add nodes to the storage
        let _ = disk_storage
            .add_nodes(&[vec![1.0, 2.0], vec![3.0, 4.0]])
            .unwrap();

        // Retrieve all node indexes and verify
        let node_indexes = disk_storage.get_all_node_indexes().unwrap();
        assert_eq!(node_indexes.len(), 2);
        assert!(node_indexes.contains(&1));
        assert!(node_indexes.contains(&2));
    }
}
