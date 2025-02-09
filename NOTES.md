## Objective of project

1. Deep dive into vector dbs
2. Get more Rust practice
3. Understand Vamana index and disk storage of index
4. Try out benchmarking, see [qdrant](https://qdrant.tech/benchmarks/)
5. Explore if possible to try out io_uring

## TODO

- Implement insert/delete based on Fresh-DiskANN
- Try out io_uring experiments?
- io_uring should help with
- Dig into <https://github.com/infrawhispers/anansi>
  - How is RocksDB used?
- qdrant benchmarking
- How would this be sharded/scaled across machines?

## Questions

- In-mem component shouldn't have all the data. What happens if we need to access data in disk while we're operating on the disk files? Is there a lock
  - Can reference LSM tree
- f32 vs f64
- how did simsimd extend f32::
  
## Done

[x] Explore disk storage representation
[x] Read Fresh-DiskANN, Filtered-DiskANN
[x] Arbitrary vector dimension