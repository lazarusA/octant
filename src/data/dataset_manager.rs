//! Manages multiple independently selected datasets.

use std::collections::HashMap;

use super::dataset::Dataset;

#[derive(Default)]
pub struct DatasetManager {
    datasets: HashMap<String, Dataset>,
}

impl DatasetManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, dataset: Dataset) {
        self.datasets.insert(dataset.id.clone(), dataset);
    }

    pub fn remove(&mut self, dataset_id: &str) -> Option<Dataset> {
        self.datasets.remove(dataset_id)
    }

    pub fn get(&self, dataset_id: &str) -> Option<&Dataset> {
        self.datasets.get(dataset_id)
    }

    pub fn get_mut(&mut self, dataset_id: &str) -> Option<&mut Dataset> {
        self.datasets.get_mut(dataset_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Dataset> {
        self.datasets.values()
    }

    pub fn contains(&self, dataset_id: &str) -> bool {
        self.datasets.contains_key(dataset_id)
    }

    pub fn len(&self) -> usize {
        self.datasets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.datasets.is_empty()
    }

    pub fn add_variable(&mut self, dataset_id: &str, variable: impl Into<String>) -> bool {
        if let Some(dataset) = self.datasets.get_mut(dataset_id) {
            dataset.add_variable(variable);
            true
        } else {
            false
        }
    }

    pub fn remove_variable(&mut self, dataset_id: &str, variable: &str) -> bool {
        if let Some(dataset) = self.datasets.get_mut(dataset_id) {
            dataset.remove_variable(variable);
            true
        } else {
            false
        }
    }
}
