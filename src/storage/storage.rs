use std::collections::{HashMap, HashSet};

use crate::prelude::*;

use crate::graph::Node;

pub trait GraphStorage {
    fn add_nodes(&mut self, data: &[Vec<f32>]) -> Result<Vec<u32>>;
    fn get_node(&self, node_id: u32) -> Result<Node>;
    fn set_connections(&mut self, node_index: u32, connections: &HashSet<u32>) -> Result<()>;
    fn get_random_node(&self) -> Option<Node>;
    fn get_all_node_indexes(&self) -> Result<Vec<u32>>;
    fn get_all_nodes(&self) -> Result<HashMap<u32, Node>>;
}

pub trait DataStore {
    fn add_data(&mut self, node_id: u32, data: String) -> Result<()>;
    fn get_data(&self, node_id: u32) -> Option<String>;
}
