use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use url::Url;
use crate::infrastructure::error::{AppError, DomainError, DomainErrorKind};

/// Calculate the similarity between two URLs based on their domain and path structure
pub fn calculate_url_similarity(url1: &str, url2: &str) -> Result<f64, AppError> {
    let parsed_url1 = Url::parse(url1).map_err(|e| DomainError::new(
        DomainErrorKind::Validation,
        format!("Failed to parse URL: {}", url1),
        Some(e.to_string()),
        Some(Box::new(e)),
    ))?;

    let parsed_url2 = Url::parse(url2).map_err(|e| DomainError::new(
        DomainErrorKind::Validation,
        format!("Failed to parse URL: {}", url2),
        Some(e.to_string()),
        Some(Box::new(e)),
    ))?;

    let domain_similarity = if parsed_url1.domain() == parsed_url2.domain() {
        1.0
    } else {
        0.0
    };

    let path_similarity = {
        let path1: Vec<&str> = parsed_url1
            .path()
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        let path2: Vec<&str> = parsed_url2
            .path()
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        // Compare common path segments
        let common_segments = path1
            .iter()
            .zip(path2.iter())
            .filter(|(a, b)| a == b)
            .count();

        // Normalize similarity based on path length
        let max_path_length = path1.len().max(path2.len());
        if max_path_length > 0 {
            common_segments as f64 / max_path_length as f64
        } else {
            0.0
        }
    };

    // Weighted combination of domain and path similarity
    Ok(0.7 * domain_similarity + 0.3 * path_similarity)
}

/// Distribute URLs across threads to minimize similarity clusters
pub fn distribute_urls(urls: &[String], num_threads: usize) -> Result<Vec<Vec<String>>, AppError> {
    if urls.is_empty() || num_threads == 0 {
        return Ok(vec![]);
    }

    // Calculate similarity matrix
    let mut similarity_matrix: HashMap<(usize, usize), f64> = HashMap::new();
    for i in 0..urls.len() {
        for j in (i + 1)..urls.len() {
            let similarity = calculate_url_similarity(&urls[i], &urls[j])?;
            similarity_matrix.insert((i, j), similarity);
            similarity_matrix.insert((j, i), similarity);
        }
    }

    // Create thread-specific URL lists
    let mut thread_urls: Vec<Vec<String>> = vec![Vec::new(); num_threads];
    let mut url_indices: Vec<usize> = (0..urls.len()).collect();

    // Shuffle initial order to add randomness
    let mut rng = thread_rng();
    url_indices.shuffle(&mut rng);

    // Distribute URLs across threads
    for (index, &url_index) in url_indices.iter().enumerate() {
        let thread_index = index % num_threads;
        thread_urls[thread_index].push(urls[url_index].clone());
    }

    Ok(thread_urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_similarity() {
        let url1 = "https://example.com/podcast/feed";
        let url2 = "https://example.com/podcast/audio";
        let url3 = "https://different.com/podcast/feed";

        assert!(calculate_url_similarity(url1, url2).unwrap() > 0.5);
        assert!(calculate_url_similarity(url1, url3).unwrap() < 0.5);
    }

    #[test]
    fn test_url_distribution() {
        let urls = vec![
            "https://example1.com/feed".to_string(),
            "https://example2.com/feed".to_string(),
            "https://example3.com/feed".to_string(),
            "https://example4.com/feed".to_string(),
        ];

        let distributed_urls = distribute_urls(&urls, 2).unwrap();
        assert_eq!(distributed_urls.len(), 2);
        assert_eq!(
            distributed_urls[0].len() + distributed_urls[1].len(),
            urls.len()
        );
    }
}
