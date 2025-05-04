use std::{
    fs::{self, File},
    io::{self, BufRead},
};

use polars::{
    export::num::ToPrimitive,
    prelude::{LazyFrame, ScanArgsParquet},
    series::Series,
};
use vdb::prelude::Result;

pub(crate) fn read_query_vector() -> Result<Vec<f32>> {
    let file = File::open("dataset/query.txt")?;
    let reader = io::BufReader::new(file);

    let mut floats = Vec::with_capacity(1536);

    for line in reader.lines() {
        let line = line?;
        floats.push(line.parse::<f32>().unwrap());
    }

    Ok(floats)
}

pub(crate) fn read_dataset(
    dataset_path: &str,
    ingest_files: i64,
) -> impl Iterator<Item = Vec<(Vec<f32>, String)>> {
    let mut paths = Vec::new();
    let mut entries = fs::read_dir(dataset_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<std::result::Result<Vec<_>, io::Error>>()
        .unwrap();

    entries.sort();

    for path in entries {
        if path.is_file() && path.extension().unwrap() == "parquet" {
            if ingest_files >= 0 && paths.len() == ingest_files as usize {
                break;
            }
            paths.push(path.to_str().unwrap().to_string());
            println!("{:?}", paths);
        }
    }

    let vec_iter = paths
        .into_iter()
        .map(|path: String| -> _ { read_datafile(&path) });
    vec_iter
}

fn read_datafile(file: &str) -> Vec<(Vec<f32>, String)> {
    let args = ScanArgsParquet::default();
    let df = LazyFrame::scan_parquet(file, args)
        .unwrap()
        .collect()
        .unwrap();
    let vector_column = df.column("openai").unwrap().list().unwrap();
    let text_column = df.column("text").unwrap().utf8().unwrap();
    fn parse_vector(arr: Option<Series>) -> Vec<f32> {
        let series = arr.ok_or("series not found").unwrap();
        let arr_value = series.f64().unwrap();
        let single_vector: Vec<f32> = arr_value.into_iter().filter_map(|x| x?.to_f32()).collect();
        single_vector
    }
    let vecs = vector_column.into_iter().map(parse_vector);
    let text: Vec<String> = text_column
        .into_iter()
        .filter_map(|x| x)
        .map(|f| f.to_owned())
        .collect();
    vecs.zip(text).collect()
}
