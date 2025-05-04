use std::{
    collections::{HashMap, HashSet},
    default, io,
};

use rand::Rng;

use crate::graph::Node;
use crate::{prelude::*, Error};

use super::{storage::DataStore, IndexStore};

#[derive(Default)]
pub struct InMemStorage {
    nodes: Vec<Node>,
    data: HashMap<u32, String>,
}

impl IndexStore for InMemStorage {
    fn add_nodes(&mut self, data: &[Vec<f32>]) -> Result<Vec<u32>> {
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

    fn get_node(&self, node_id: u32) -> Result<Node> {
        self.nodes
            .get(node_id as usize)
            .cloned()
            .ok_or_else(|| Error::InvalidInput("Node not found".to_owned()))
    }

    fn set_connections(&mut self, node_index: u32, connections: &HashSet<u32>) -> Result<()> {
        if let Some(node) = self.nodes.get_mut(node_index as usize) {
            node.connected = connections.clone();
            Ok(())
        } else {
            Err(Error::InvalidInput("Node not found".to_owned()))
        }
    }

    fn get_random_node(&self) -> Option<Node> {
        return self.nodes.get(0).cloned();
    }

    fn get_all_node_indexes(&self) -> Result<Vec<u32>> {
        Ok((0..self.nodes.len() as u32).collect())
    }

    fn get_all_nodes(&self) -> Result<HashMap<u32, Node>> {
        Ok(self
            .nodes
            .iter()
            .map(|node| (node.id, node.clone()))
            .collect())
    }
}

impl DataStore for InMemStorage {
    fn add_data(&mut self, node_id: u32, data: String) -> Result<()> {
        self.data.insert(node_id, data);
        Ok(())
    }

    fn get_data(&self, node_id: u32) -> Option<String> {
        self.data.get(&node_id).cloned()
    }
}
