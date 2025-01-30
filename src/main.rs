#![warn(unused_extern_crates)]
use polars::prelude::*;

mod constant;
mod graph;
mod storage;
fn main() {
    graph::vamana::init();
    // let res = read_dataset(
    //     "dataset/dbpedia-entities-openai-1M/train-00000-of-00026-3c7b99d1c7eda36e.parquet",
    // );
    // println!("{:?}", res);
}

fn read_dataset(dataset_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let args = ScanArgsParquet::default();
    let lf = LazyFrame::scan_parquet(dataset_file, args).unwrap();

    let schema = lf.schema().unwrap();
    print!("{:?}", schema);

    Ok(())
}
