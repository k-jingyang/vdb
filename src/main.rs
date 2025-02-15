#![warn(unused_extern_crates)]
use std::time::Duration;

use constant::{MAX_NEIGHBOUR_COUNT, VECTOR_DIMENSION};
use polars::{export::num::ToPrimitive, prelude::*};
use tokio::runtime;
mod constant;

// 1000 vectors of 1536 dimensions
// in-mem indexing: 19ms
// disk indexing: 22s
// difference is 1000x
// after opimitisation of single read:
// disk indexing: 227ms
//

// Read 1000000 vectors of dimension: 1536
// In-mem graph::new took 324.854365107s
// In-mem graph::index took 46.901828688s
// Disk graph::new took 194.129564367s
// Disk graph::index took 1635.210504063s
fn main() {
    let disk = vdb::storage::NaiveDisk::new(
        VECTOR_DIMENSION,
        MAX_NEIGHBOUR_COUNT,
        "disk.index",
        "disk.free",
    )
    .unwrap();
    let fresh_disk =
        vdb::storage::FreshDisk::new(2, MAX_NEIGHBOUR_COUNT, "disk.index", "disk.free").unwrap();
    let in_mem = vdb::storage::InMemStorage::new();

    vdb::vamana::debug(100, 0.0..2000.0, Box::new(fresh_disk));
    println!("Done debug");
    std::thread::sleep(Duration::from_secs(100));
}

fn run_dataset_test() {
    const MAX_NEIGHBOUR_COUNT: u8 = 5;

    // vdb::vamana::init();
    let res = read_dataset("dataset/dbpedia-entities-openai-1M/data/", -1)
        .unwrap()
        .to_vec();
    println!("Read {} vectors of dimension: {}", res.len(), res[0].len());

    let start = std::time::Instant::now();
    let mut in_mem_graph = vdb::graph::Graph::new(
        &res,
        2,
        MAX_NEIGHBOUR_COUNT,
        Box::new(vdb::storage::InMemStorage::new()),
    )
    .unwrap();
    println!("In-mem graph::new took {:?}", start.elapsed());

    let start = std::time::Instant::now();
    in_mem_graph.index(1.2).unwrap();
    println!("In-mem graph::index took {:?}", start.elapsed());

    let start = std::time::Instant::now();
    in_mem_graph.index(1.2).unwrap();
    let mut disk_graph = vdb::graph::Graph::new(
        &res,
        2,
        MAX_NEIGHBOUR_COUNT,
        Box::new(
            vdb::storage::NaiveDisk::new(
                res[0].len() as u16,
                MAX_NEIGHBOUR_COUNT,
                "disk.index",
                "disk.free",
            )
            .unwrap(),
        ),
    )
    .unwrap();
    println!("Disk graph::new took {:?}", start.elapsed());

    let start = std::time::Instant::now();
    disk_graph.index(1.2).unwrap();
    println!("Disk graph::index took {:?}", start.elapsed());
}

fn read_dataset(
    dataset_path: &str,
    ingest_files: i64,
) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dataset_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap() == "parquet" {
            if ingest_files >= 0 && paths.len() == ingest_files as usize {
                break;
            }
            paths.push(path.to_str().unwrap().to_string());
        }
    }

    let args = ScanArgsParquet::default();

    let mut result = Vec::new();
    for path in paths {
        let df = LazyFrame::scan_parquet(path.as_str(), args.clone())?.collect()?;
        let list_column = df.column("openai")?.list()?;
        for arr in list_column.into_iter() {
            let series = arr.ok_or("series not found")?;
            let arr_value = series.f64()?;
            let vec: Vec<f32> = arr_value.into_iter().filter_map(|x| x?.to_f32()).collect();
            result.push(vec);
        }
    }

    Ok(result)
}
