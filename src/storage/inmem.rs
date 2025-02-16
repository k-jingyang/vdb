use std::{
    collections::{HashMap, HashSet},
    io,
};

use rand::Rng;

use crate::graph::Node;

use super::GraphStorage;

pub struct InMemStorage {
    nodes: Vec<Node>,
}

impl InMemStorage {
    pub fn new() -> Self {
        InMemStorage { nodes: Vec::new() }
    }
}

impl GraphStorage for InMemStorage {
    fn add_nodes(&mut self, data: &[Vec<f32>]) -> io::Result<Vec<u32>> {
        let mut node_ids = Vec::new();
        for vector in data {
            let node_id = self.nodes.len() as u32;
            self.nodes.push(Node {
                id: node_id,
                vector: vector.clone(),
                connected: HashSet::new(),
            });
            node_ids.push(node_id);
        }
        Ok(node_ids)
    }

    fn get_node(&self, node_id: u32) -> io::Result<Node> {
        self.nodes
            .get(node_id as usize)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Node not found"))
    }

    fn set_connections(&mut self, node_index: u32, connections: &HashSet<u32>) -> io::Result<()> {
        if let Some(node) = self.nodes.get_mut(node_index as usize) {
            println!("before: {:?}, connections: {:?}", node, connections);
            node.connected = connections.clone();
            println!("after: {:?}", node);
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Node not found"))
        }
    }

    fn get_random_node(&self) -> Option<Node> {
        return self.nodes.get(0).cloned();
    }

    fn get_all_node_indexes(&self) -> Vec<u32> {
        (0..self.nodes.len() as u32).collect()
    }

    fn get_all_nodes(&self) -> HashMap<u32, Node> {
        self.nodes
            .iter()
            .map(|node| (node.id, node.clone()))
            .collect()
    }
}
