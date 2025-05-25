use vdb::{InMemStorage, IndexStore};

use crate::{data, MAX_NEIGHBOUR_COUNT};

// index_dbpedia indexes the dbpedia dataset using index_storage_type. The number of files to read from the dataset can be specified with dataset_files. -1 to load all files (note that this will incur a huge indexing time)
pub(super) fn index_dbpedia(index_storage: Box<dyn IndexStore>, dataset_files: i64) -> vdb::Graph {
    let res = data::read_dataset("dataset/dbpedia-entities-openai-1M/data/", dataset_files);
    let start = std::time::Instant::now();
    let index_name = index_storage.get_name();
    let mut graph = vdb::graph::Graph::new(
        res,
        5,
        MAX_NEIGHBOUR_COUNT,
        index_storage,
        Box::new(InMemStorage::default()), // TODO: Can provide other implementations
    )
    .unwrap();
    println!("{} graph::new took {:?}", index_name, start.elapsed());

    let start = std::time::Instant::now();
    graph.index(1.0).unwrap();
    graph.index(1.0).unwrap();
    println!("{} graph::index took {:?}", index_name, start.elapsed());
    graph
}

pub(super) fn query_dbpedia_index(graph: &vdb::Graph, query: &[f32], k: usize) -> Vec<String> {
    let closest_k_nodes = graph.greedy_search_random_start(query, k, 10);
    let mut result = Vec::with_capacity(closest_k_nodes.0.len());
    for node_index in closest_k_nodes.0 {
        let data = graph.data_store.get_data(node_index);
        if let Some(content) = data {
            result.push(content);
        }
    }
    result
}
