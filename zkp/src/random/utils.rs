use rand::Rng;

// Generate a random vector of length n, with the sum of all elements equal to m.
pub fn generate_random_vec<R: Rng>(rng: &mut R, n: usize, m: usize) -> Vec<usize> {
    let mut vec = vec![0; n];
    for _ in 0..m {
        let idx = rng.gen_range(0..n);
        vec[idx] += 1;
    }
    vec
}
