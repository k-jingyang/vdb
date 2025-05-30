## Objective of project

1. Deep dive into vector dbs
2. Get more Rust practice
3. Understand Vamana index and disk storage of index
4. Try out benchmarking, see [qdrant](https://qdrant.tech/benchmarks/)
5. Explore if possible to try out io_uring

## TODO

- Implement search
- qdrant benchmarking
- Implement delete based on Fresh-DiskANN
- Try out io_uring experiments?
  - Measure syscalls before and after
- io_uring should help with
- Dig into <https://github.com/infrawhispers/anansi>
  - How is RocksDB used?
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
[x] Implemented Fresh-DiskANN for insert
[x] Tested Fresh-DiskANN for 1 million vectors
[x] Identify fresh disk flushing making PC sluggish
[x] Read LSM tree implementation to see background process implementation