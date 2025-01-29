use std::io;

use crate::graph::{Graph, Node};

pub trait GraphStorage {
    fn add_nodes(&self, nodes: &[Node]) -> io::Result<()>;
    fn add_connections(&self, connections: &[(Node, Node)]) -> io::Result<()>;
}
