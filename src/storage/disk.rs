use rand::Rng;

use crate::graph::Node;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Error, ErrorKind};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
};

use super::storage::GraphStorage;
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
// 1. Figure out how to do metadata storage?
// 2. log based input instead
// 3. decouple node_id from node index
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
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut index_file = BufWriter::new(File::create(index_path)?);

        // Write metadata to index file
        index_file.write_all(&dimensions.to_be_bytes())?;
        index_file.write_all(&max_neighbor_count.to_be_bytes())?;
        index_file.write_all(&(0u32).to_be_bytes())?;

        Ok(NaiveDisk {
            dimensions: dimensions,
            max_neighbour_count: max_neighbor_count,
            next_node_index: 0,
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
        self.index_metadata_size() as u64 + (node_index as usize * self.index_node_size()) as u64
    }

    fn node_connections_offset(&self, node_index: u32) -> u64 {
        self.node_offset(node_index)
            + std::mem::size_of::<u32>() as u64
            + (self.dimensions as usize * std::mem::size_of::<f32>()) as u64
    }

    pub(crate) fn set_node(&mut self, node: &Node) -> io::Result<()> {
        println!("Writing node: {}", node.id);
        let mut f = OpenOptions::new().write(true).open(&self.index_path)?;
        f.seek(SeekFrom::Current(self.index_metadata_size() as i64))?;
        f.seek(SeekFrom::Current(
            (node.id as usize * self.index_node_size()) as i64,
        ))?;

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
            index_file.write_all(&u32::MAX.to_be_bytes())?;
        }
        Ok(())
    }
}

impl GraphStorage for NaiveDisk {
    fn set_connections(&mut self, node_index: u32, connections: &HashSet<u32>) -> io::Result<()> {
        if connections.len() > self.max_neighbour_count as usize {
            return Err(Error::new(ErrorKind::Other, "max connections reached"));
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
            index_file.write_all(&u32::MAX.to_be_bytes())?;
        }
        Ok(())
    }

    fn add_nodes(&mut self, data: &[Vec<f32>]) -> io::Result<Vec<u32>> {
        let mut created_node_indices: Vec<u32> = Vec::new();

        // jump to next node offset
        let mut f = OpenOptions::new().write(true).open(&self.index_path)?;
        f.seek(SeekFrom::Current(self.index_metadata_size() as i64))?;
        f.seek(SeekFrom::Current(
            (self.next_node_index as usize * self.index_node_size()) as i64,
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

    fn get_node(&self, node_index: u32) -> io::Result<Node> {
        let mut index_file = File::open(&self.index_path)?;

        index_file.seek(SeekFrom::Current(
            self.index_metadata_size() as i64
                + (node_index as usize * self.index_node_size()) as i64,
        ))?;

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
            if neighbor_index != u32::MAX {
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

    fn get_all_node_indexes(&self) -> Vec<u32> {
        // TODO: need to exclude free list nodes
        let mut node_indexes = Vec::new();
        for i in 0..self.next_node_index {
            node_indexes.push(i);
        }
        node_indexes
    }

    fn get_all_nodes(&self) -> HashMap<u32, Node> {
        let mut all_nodes: HashMap<u32, Node> = HashMap::new();
        for node_index in self.get_all_node_indexes() {
            all_nodes.insert(node_index, self.get_node(node_index).unwrap());
        }
        all_nodes
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

        assert_eq!(ids, vec![0, 1]);
        disk_storage
            .set_connections(0, &HashSet::from([1u32]))
            .unwrap();
        disk_storage
            .set_connections(1, &HashSet::from([0u32]))
            .unwrap();

        // Retrieve nodes and verify
        let retrieved_node1 = disk_storage.get_node(0).unwrap();
        let retrieved_node2 = disk_storage.get_node(1).unwrap();

        assert_eq!(0, retrieved_node1.id);
        assert_eq!(vec![1.0, 2.0], retrieved_node1.vector);
        assert_eq!(HashSet::from([1]), retrieved_node1.connected);

        assert_eq!(1, retrieved_node2.id);
        assert_eq!(vec![3.0, 4.0], retrieved_node2.vector);
        assert_eq!(HashSet::from([0]), retrieved_node2.connected);
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
}
