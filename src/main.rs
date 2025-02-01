#![warn(unused_extern_crates)]
use polars::{
    export::{arrow::array::ListArray, num::ToPrimitive},
    prelude::*,
};

fn main() {
    // vdb::vamana::init();
    let res = read_dataset(
        "dataset/dbpedia-entities-openai-1M/train-00000-of-00026-3c7b99d1c7eda36e.parquet",
    )
    .unwrap();

    const MAX_NEIGHBOUR_COUNT: u8 = 5;

    let mut in_mem_graph = vdb::graph::Graph::new(
        &res,
        2,
        MAX_NEIGHBOUR_COUNT,
        Box::new(vdb::storage::InMemStorage::new()),
    )
    .unwrap();

    println!("created graph");
    in_mem_graph.index(1.2).unwrap();
}

fn read_dataset(dataset_file: &str) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    let args = ScanArgsParquet::default();

    let df = LazyFrame::scan_parquet(dataset_file, ScanArgsParquet::default())?.collect()?;
    // Assuming the column name is "list_column"
    let list_column = df.column("openai")?.list()?;

    // Initialize the result vector
    let mut result: Vec<Vec<f32>> = Vec::new();

    // Iterate over the list column
    for arr in list_column.into_iter() {
        let series = arr.ok_or("series not found")?;
        let arr_value = series.f64()?;
        let vec: Vec<f32> = arr_value
            .into_iter()
            .map(|opt| opt.unwrap_or(0.0).to_f32().unwrap_or(0.0))
            .collect();
        result.push(vec);
    }

    Ok(result)
}
