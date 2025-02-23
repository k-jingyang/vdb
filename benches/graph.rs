use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vdb::graph;

fn bench_index(c: &mut Criterion) {
    const SIZE: usize = 100;
    const VALUE_RANGE: std::ops::Range<f32> = 0.0..2000.0;
    const DIMENSION: u16 = 2;
    const MAX_NEIGHBOUR_COUNT: u8 = 5;

    let test_vectors = vdb::vector::generate_random_vectors(SIZE, &VALUE_RANGE, DIMENSION as usize);

    c.bench_function("[in-mem] create and index graph", |b| {
        b.iter(|| {
            create_and_index_graph(
                black_box(&test_vectors),
                || Box::new(vdb::storage::InMemStorage::new()),
                MAX_NEIGHBOUR_COUNT,
            )
        })
    });

    c.bench_function("[disk] create and index graph", |b| {
        b.iter(|| {
            create_and_index_graph(
                black_box(&test_vectors),
                || {
                    Box::new(
                        vdb::storage::NaiveDisk::new(
                            DIMENSION,
                            MAX_NEIGHBOUR_COUNT,
                            "disk.index",
                            "disk.free",
                        )
                        .unwrap(),
                    )
                },
                MAX_NEIGHBOUR_COUNT,
            )
        })
    });
}

fn bench_query(c: &mut Criterion) {
    const SIZE: usize = 100;
    const VALUE_RANGE: std::ops::Range<f32> = 0.0..2000.0;
    const DIMENSION: u16 = 2;
    const MAX_NEIGHBOUR_COUNT: u8 = 5;

    let test_vectors = vdb::vector::generate_random_vectors(SIZE, &VALUE_RANGE, DIMENSION as usize);
    let mut in_mem_graph = graph::Graph::new(
        &test_vectors,
        2,
        MAX_NEIGHBOUR_COUNT,
        Box::new(vdb::storage::InMemStorage::new()),
    )
    .unwrap();
    in_mem_graph.index(1.2).unwrap();

    c.bench_function("[in-mem] query", |b| {
        b.iter(|| in_mem_graph.greedy_search(1, &[1000.0f32, 1000.0f32], 3, 10));
    });

    let mut disk_graph = graph::Graph::new(
        &test_vectors,
        2,
        MAX_NEIGHBOUR_COUNT,
        Box::new(
            vdb::storage::NaiveDisk::new(DIMENSION, MAX_NEIGHBOUR_COUNT, "disk.index", "disk.free")
                .unwrap(),
        ),
    )
    .unwrap();

    disk_graph.index(1.2).unwrap();

    c.bench_function("[disk] query", |b| {
        b.iter(|| disk_graph.greedy_search(1, &[1000.0f32, 1000.0f32], 3, 10));
    });
}

fn create_and_index_graph(
    test_vectors: &Vec<Vec<f32>>,
    storage_factory: fn() -> Box<dyn vdb::storage::GraphStorage>,
    max_neighbour_count: u8,
) {
    const R: usize = 2;

    let mut graph =
        graph::Graph::new(&test_vectors, R, max_neighbour_count, storage_factory()).unwrap();

    graph.index(1.0).unwrap();
}

criterion_group!(benches, bench_index, bench_query);
criterion_main!(benches);
