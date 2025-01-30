use std::{collections::HashSet, io};

use crate::graph::{Graph, Node};

pub trait GraphStorage {
    // fn add_connections(&self, connections: &[(u32, u32)]) -> io::Result<()>;
    fn add_nodes(&mut self, data: &[Vec<f32>]) -> io::Result<Vec<u32>>;
    fn get_node(&self, node_id: u32) -> io::Result<Node>;
    fn set_connections(&mut self, node_index: u32, connections: &HashSet<u32>) -> io::Result<()>;
    fn get_random_node(&self) -> Option<Node>;
    fn get_all_node_indexes(&self) -> Vec<u32>;
    fn get_all_nodes(&self) -> Vec<Node>;
}
