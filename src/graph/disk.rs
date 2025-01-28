use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
};

use crate::{constant::VECTOR_DIMENSION, graph::graph::Node};

use super::graph::Graph;

// disk layout
//
// .index file:
// [metadata][nodes]
//
// where [metadata]:
// [dim][neighbor_count]
// [u16][      u8       ]
//
// where [nodes]:
// [node_id][.data file offset]
// [  u32  ][     u64         ]
//
// .data file:
// [   vector   ][    neighbor indexes     ]
// [ f32 * dim  ][u32 * neighbor_count     ]
//
// TODO: Figure out how to do metadata storage, we can't update .data file in place because any updates will result
// in the update of all the node offsets after that node
//
pub(super) fn write_to_disk(db: &Graph, index_path: &str, data_path: &str) -> io::Result<()> {
    let mut index_file = BufWriter::new(File::create(index_path)?);
    let mut data_file = BufWriter::new(File::create(data_path)?);

    // Write metadata to index file
    index_file.write_all(&(VECTOR_DIMENSION as u16).to_be_bytes())?; // dimension

    // Calculate max number of neighbors
    let max_neighbors = db
        .nodes
        .iter()
        .map(|node| node.connected.len())
        .max()
        .unwrap_or(0);
    index_file.write_all(&(max_neighbors as u8).to_be_bytes())?; //  neighbor_count

    // Write nodes to data file and record offsets in index file
    for node in &db.nodes {
        // Write node_id and offset to index file
        let offset = data_file.seek(SeekFrom::Current(0))?;
        index_file.write_all(&(node.id as u32).to_be_bytes())?;
        index_file.write_all(&(offset as u64).to_be_bytes())?;

        // Write vector to data file
        for &value in &node.vector {
            data_file.write_all(&value.to_be_bytes())?;
        }

        // Write neighbor indices to data file
        let mut neighbor_indices: Vec<u32> = node.connected.iter().map(|&x| x as u32).collect();
        neighbor_indices.resize(max_neighbors, u32::MAX); // Pad if needed
        for &id in &neighbor_indices {
            data_file.write_all(&id.to_be_bytes())?;
        }
    }

    Ok(())
}
pub(super) fn load_from_disk(index_path: &str, data_path: &str) -> io::Result<Graph> {
    let mut index_file = File::open(index_path)?;
    let mut data_file = File::open(data_path)?;

    // Read metadata from index file
    let mut dim_bytes = [0u8; 2];
    index_file.read_exact(&mut dim_bytes)?;
    let dimension = u16::from_be_bytes(dim_bytes) as usize;
    assert_eq!(dimension, VECTOR_DIMENSION, "Dimension mismatch");

    let mut neighbor_count_bytes = [0u8; 1];
    index_file.read_exact(&mut neighbor_count_bytes)?;
    let neighbor_count = neighbor_count_bytes[0] as usize;

    // Read node entries from index file
    let mut nodes = Vec::new();
    while let Ok(()) = (|| -> io::Result<()> {
        let mut node_id_bytes = [0u8; 4];
        let mut offset_bytes = [0u8; 8];

        index_file.read_exact(&mut node_id_bytes)?;
        index_file.read_exact(&mut offset_bytes)?;

        let node_id = u32::from_be_bytes(node_id_bytes) as u32;
        let offset = u64::from_be_bytes(offset_bytes);

        // Seek to position in data file
        data_file.seek(SeekFrom::Start(offset))?;

        // Read vector
        let mut vector = [0.0f32; VECTOR_DIMENSION];
        for value in &mut vector {
            let mut bytes = [0u8; 4];
            data_file.read_exact(&mut bytes)?;
            *value = f32::from_be_bytes(bytes);
        }

        // Read neighbor indices
        let mut connected = HashSet::new();
        for _ in 0..neighbor_count {
            let mut neighbor_bytes = [0u8; 4];
            data_file.read_exact(&mut neighbor_bytes)?;
            let neighbor_index = u32::from_be_bytes(neighbor_bytes);

            // Ignore padding
            if neighbor_index != u32::MAX {
                connected.insert(neighbor_index as usize);
            }
        }

        nodes.push(Node {
            id: node_id,
            vector: vector,
            connected: connected,
        });

        Ok(())
    })() {}
    // Continue reading nodes

    Ok(Graph { nodes })
}

#[cfg(test)]
mod tests {
    use crate::graph::disk;

    use super::*;
    use std::{env, fs};

    #[test]
    fn test_graph_serialization() -> io::Result<()> {
        // Create temporary directory for test files
        let temp_dir = env::temp_dir();
        let index_path = temp_dir.as_path().join("test.index");
        let data_path = temp_dir.as_path().join("test.data");

        // Create a test graph
        let mut original_graph = Graph { nodes: Vec::new() };

        // Add some test nodes
        let node1 = Node {
            id: 100,
            vector: [1.0, 2.0], // Assuming VECTOR_DIMENSION = 3
            connected: [1, 2].iter().cloned().collect(),
        };

        let node2 = Node {
            id: 200,
            vector: [4.0, 5.0],
            connected: [0, 2].iter().cloned().collect(),
        };

        let node3 = Node {
            id: 300,
            vector: [7.0, 8.0],
            connected: [0, 1].iter().cloned().collect(),
        };

        original_graph.nodes.push(node1);
        original_graph.nodes.push(node2);
        original_graph.nodes.push(node3);

        // Save the graph
        disk::write_to_disk(
            &original_graph,
            index_path.to_str().unwrap(),
            data_path.to_str().unwrap(),
        )
        .unwrap();

        // Load the graph back
        let loaded_graph =
            disk::load_from_disk(index_path.to_str().unwrap(), data_path.to_str().unwrap())?;

        // Assert the graphs are equal
        assert_eq!(original_graph.nodes.len(), loaded_graph.nodes.len());

        for (original_node, loaded_node) in
            original_graph.nodes.iter().zip(loaded_graph.nodes.iter())
        {
            // Compare vectors
            assert_eq!(original_node.vector, loaded_node.vector);

            // Compare connected sets
            assert_eq!(original_node.connected, loaded_node.connected);
        }

        // Verify file sizes
        let index_metadata_size = std::mem::size_of::<u16>() + std::mem::size_of::<u8>();
        let index_entry_size = std::mem::size_of::<u32>() + std::mem::size_of::<u64>();
        let expected_index_size =
            index_metadata_size + (index_entry_size * original_graph.nodes.len());

        let vector_size = VECTOR_DIMENSION * std::mem::size_of::<f32>();
        let max_neighbors = 2; // Based on our test data
        let neighbors_size = max_neighbors * std::mem::size_of::<u32>();
        let expected_data_size = (vector_size + neighbors_size) * original_graph.nodes.len();

        assert_eq!(
            fs::metadata(&index_path)?.len() as usize,
            expected_index_size
        );
        assert_eq!(fs::metadata(&data_path)?.len() as usize, expected_data_size);

        Ok(())
    }

    #[test]
    fn test_empty_graph() -> io::Result<()> {
        let temp_dir = env::temp_dir();
        let index_path = temp_dir.as_path().join("empty.index");
        let data_path = temp_dir.as_path().join("empty.data");

        let empty_graph = Graph { nodes: Vec::new() };

        disk::write_to_disk(
            &empty_graph,
            index_path.to_str().unwrap(),
            data_path.to_str().unwrap(),
        )?;

        let loaded_graph =
            disk::load_from_disk(index_path.to_str().unwrap(), data_path.to_str().unwrap())?;

        assert_eq!(empty_graph.nodes.len(), loaded_graph.nodes.len());
        assert_eq!(loaded_graph.nodes.len(), 0);

        Ok(())
    }
}
