use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct ProjectionResult {
    pub positions: Vec<(f32, f32)>,
    pub normalized_stress: f32,
}

pub fn project_to_2d(embeddings: &[Vec<f32>]) -> ProjectionResult {
    if embeddings.is_empty() {
        return ProjectionResult {
            positions: Vec::new(),
            normalized_stress: 0.0,
        };
    }

    if embeddings.len() == 1 {
        return ProjectionResult {
            positions: vec![(0.0, 0.0)],
            normalized_stress: 0.0,
        };
    }

    let distances = pairwise_cosine_distances(embeddings);
    let mut positions = initial_positions(embeddings.len());
    let mut best_positions = positions.clone();
    let mut best_stress = total_stress(&positions, &distances);

    // Burn-in gives the solver room to untangle a poor random start.
    let burn_steps = 80;
    let settle_steps = 160;
    let mut learning_rate = 0.12;
    for step in 0..(burn_steps + settle_steps) {
        iterate_mds_step(&mut positions, &distances, learning_rate);
        let stress = total_stress(&positions, &distances);
        if stress < best_stress {
            best_stress = stress;
            best_positions.clone_from(&positions);
        }

        learning_rate *= if step < burn_steps { 0.985 } else { 0.992 };
    }

    recenter(&mut best_positions);
    ProjectionResult {
        normalized_stress: normalized_stress(best_stress, &distances),
        positions: best_positions,
    }
}

fn pairwise_cosine_distances(embeddings: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let norms = embeddings
        .iter()
        .map(|embedding| embedding.iter().map(|value| value * value).sum::<f32>().sqrt())
        .collect::<Vec<_>>();

    let mut distances = vec![vec![0.0; embeddings.len()]; embeddings.len()];
    for left in 0..embeddings.len() {
        for right in (left + 1)..embeddings.len() {
            let distance = cosine_distance(&embeddings[left], norms[left], &embeddings[right], norms[right]);
            distances[left][right] = distance;
            distances[right][left] = distance;
        }
    }

    distances
}

fn cosine_distance(left: &[f32], left_norm: f32, right: &[f32], right_norm: f32) -> f32 {
    if left.is_empty() || right.is_empty() || left_norm <= f32::EPSILON || right_norm <= f32::EPSILON {
        return 1.0;
    }

    let dot = left
        .iter()
        .zip(right.iter())
        .map(|(left_value, right_value)| left_value * right_value)
        .sum::<f32>();

    (1.0 - (dot / (left_norm * right_norm)).clamp(-1.0, 1.0)).max(0.0)
}

fn initial_positions(count: usize) -> Vec<(f32, f32)> {
    let mut rng = StdRng::seed_from_u64(0x5EED_2D5A);
    (0..count)
        .map(|_| (rng.gen_range(-1.0f32..1.0f32), rng.gen_range(-1.0f32..1.0f32)))
        .collect()
}

fn iterate_mds_step(positions: &mut [(f32, f32)], target_distances: &[Vec<f32>], learning_rate: f32) {
    let mut gradients = vec![(0.0f32, 0.0f32); positions.len()];

    for index in 0..positions.len() {
        for other_index in (index + 1)..positions.len() {
            let (x, y) = positions[index];
            let (other_x, other_y) = positions[other_index];
            let delta_x = x - other_x;
            let delta_y = y - other_y;
            let current_distance = (delta_x * delta_x + delta_y * delta_y).sqrt().max(0.0001);
            let target_distance = target_distances[index][other_index].max(0.0001);
            let error_scale = (current_distance - target_distance) / current_distance;

            gradients[index].0 += delta_x * error_scale;
            gradients[index].1 += delta_y * error_scale;
            gradients[other_index].0 -= delta_x * error_scale;
            gradients[other_index].1 -= delta_y * error_scale;
        }
    }

    let normalization = (positions.len().saturating_sub(1)).max(1) as f32;
    for (position, gradient) in positions.iter_mut().zip(gradients.into_iter()) {
        position.0 -= (gradient.0 / normalization) * learning_rate;
        position.1 -= (gradient.1 / normalization) * learning_rate;
    }

    recenter(positions);
}

fn total_stress(positions: &[(f32, f32)], target_distances: &[Vec<f32>]) -> f32 {
    let mut total = 0.0;
    for left in 0..positions.len() {
        for right in (left + 1)..positions.len() {
            let projected_distance = euclidean_distance(positions[left], positions[right]);
            let target_distance = target_distances[left][right];
            total += (projected_distance - target_distance).powi(2);
        }
    }
    total
}

fn normalized_stress(stress: f32, target_distances: &[Vec<f32>]) -> f32 {
    let mut total_target_energy = 0.0;
    for left in 0..target_distances.len() {
        for right in (left + 1)..target_distances.len() {
            total_target_energy += target_distances[left][right].powi(2);
        }
    }

    if total_target_energy <= f32::EPSILON {
        return 0.0;
    }

    (stress / total_target_energy).sqrt()
}

fn euclidean_distance(left: (f32, f32), right: (f32, f32)) -> f32 {
    let dx = left.0 - right.0;
    let dy = left.1 - right.1;
    (dx * dx + dy * dy).sqrt()
}

fn recenter(positions: &mut [(f32, f32)]) {
    let count = positions.len() as f32;
    let mean_x = positions.iter().map(|(x, _)| *x).sum::<f32>() / count;
    let mean_y = positions.iter().map(|(_, y)| *y).sum::<f32>() / count;

    for position in positions {
        position.0 -= mean_x;
        position.1 -= mean_y;
    }
}

#[cfg(test)]
mod tests {
    use super::{initial_positions, pairwise_cosine_distances, project_to_2d, total_stress};

    #[test]
    fn returns_empty_projection_for_empty_input() {
        let projection = project_to_2d(&[]);
        assert!(projection.positions.is_empty());
        assert_eq!(projection.normalized_stress, 0.0);
    }

    #[test]
    fn returns_origin_for_single_embedding() {
        let projection = project_to_2d(&[vec![1.0, 0.0, 0.5]]);
        assert_eq!(projection.positions, vec![(0.0, 0.0)]);
        assert_eq!(projection.normalized_stress, 0.0);
    }

    #[test]
    fn reduces_stress_from_seeded_initial_positions() {
        let embeddings = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.96, 0.04, 0.0],
            vec![-1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
        ];

        let distances = pairwise_cosine_distances(&embeddings);
        let initial = initial_positions(embeddings.len());
        let projected = project_to_2d(&embeddings);

        assert!(
            total_stress(&projected.positions, &distances) < total_stress(&initial, &distances),
            "expected iterative MDS to reduce projection stress"
        );
        assert!(projected.normalized_stress >= 0.0);
    }

    #[test]
    fn recenters_projected_points_around_origin() {
        let embeddings = vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![-1.0, 0.0],
            vec![0.0, -1.0],
        ];

        let positions = project_to_2d(&embeddings).positions;
        let mean_x = positions.iter().map(|(x, _)| *x).sum::<f32>() / positions.len() as f32;
        let mean_y = positions.iter().map(|(_, y)| *y).sum::<f32>() / positions.len() as f32;

        assert!(mean_x.abs() < 1e-4, "expected centered x mean, got {mean_x}");
        assert!(mean_y.abs() < 1e-4, "expected centered y mean, got {mean_y}");
    }
}
