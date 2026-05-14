use rand::prelude::*;

pub fn project_to_2d(embedding: &[f32]) -> (f32, f32) {
    if embedding.is_empty() {
        return (0.0, 0.0);
    }

    let x = embedding
        .iter()
        .step_by(2)
        .enumerate()
        .map(|(index, value)| *value * ((index + 1) as f32).sin())
        .sum::<f32>();
    let y = embedding
        .iter()
        .skip(1)
        .step_by(2)
        .enumerate()
        .map(|(index, value)| *value * ((index + 1) as f32).cos())
        .sum::<f32>();

    (x, y)
}

pub fn kmeans(vectors: &[Vec<f32>], cluster_count: usize, iterations: usize) -> Vec<usize> {
    if vectors.is_empty() {
        return Vec::new();
    }
    if vectors.len() <= cluster_count {
        return (0..vectors.len()).collect();
    }

    let dimension = vectors[0].len();
    let mut rng = StdRng::seed_from_u64(42);
    let mut centroids = vectors
        .choose_multiple(&mut rng, cluster_count)
        .cloned()
        .collect::<Vec<_>>();
    let mut assignments = vec![0; vectors.len()];

    for _ in 0..iterations {
        for (index, vector) in vectors.iter().enumerate() {
            assignments[index] = centroids
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    distance(vector, left)
                        .partial_cmp(&distance(vector, right))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(centroid_index, _)| centroid_index)
                .unwrap_or(0);
        }

        let mut sums = vec![vec![0.0; dimension]; cluster_count];
        let mut counts = vec![0usize; cluster_count];
        for (assignment, vector) in assignments.iter().zip(vectors.iter()) {
            counts[*assignment] += 1;
            for (index, value) in vector.iter().enumerate() {
                sums[*assignment][index] += value;
            }
        }

        for cluster in 0..cluster_count {
            if counts[cluster] == 0 {
                centroids[cluster] = vectors[rng.gen_range(0..vectors.len())].clone();
                continue;
            }
            for value in &mut sums[cluster] {
                *value /= counts[cluster] as f32;
            }
            centroids[cluster] = sums[cluster].clone();
        }
    }

    assignments
}

fn distance(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f32>()
        .sqrt()
}
