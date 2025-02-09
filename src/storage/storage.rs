use std::{
    collections::{HashMap, HashSet},
    io,
};

use crate::graph::Node;

pub trait GraphStorage {
    fn add_nodes(&mut self, data: &[Vec<f32>]) -> io::Result<Vec<u32>>;
    fn get_node(&self, node_id: u32) -> io::Result<Node>;
    fn set_connections(&mut self, node_index: u32, connections: &HashSet<u32>) -> io::Result<()>;
    fn get_random_node(&self) -> Option<Node>;
    fn get_all_node_indexes(&self) -> Vec<u32>;
    fn get_all_nodes(&self) -> HashMap<u32, Node>;
}
