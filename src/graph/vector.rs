use rand::{thread_rng, Rng};

pub fn generate_random_vectors(
    size: usize,
    value_range: &std::ops::Range<f32>,
    dimension: usize,
) -> Vec<(Vec<f32>, String)> {
    let mut vectors: Vec<(Vec<f32>, String)> = Vec::new();
    for _ in 0..size {
        let mut arr = vec![0f32; dimension];
        for i in 0..dimension {
            let val = thread_rng().gen_range(value_range.clone());
            arr[i] = val;
        }
        vectors.push((arr.to_vec(), "".to_string()));
    }
    vectors
}
