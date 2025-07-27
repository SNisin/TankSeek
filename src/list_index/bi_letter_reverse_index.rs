use std::collections::HashMap;

use crate::file_tree::FileTree;

pub struct BiLetterIndex {
    pub index: HashMap<String, Vec<usize>>,
}
impl BiLetterIndex {
    pub fn new(tree: &FileTree) -> Self {
        let index = create_bi_letter_reverse_index(tree);
        BiLetterIndex { index }
    }

    pub fn query_word<T: AsRef<str>>(&self, word: T) -> Vec<usize> {
        // Split the query into bi-letters
        let mut bi_letters = Vec::new();
        let chars: Vec<char> = word.as_ref().chars().collect();
        for i in 0..chars.len() - 1 {
            // Create a bi-letter from the current and next character
            let bi_letter = format!("{}{}", chars[i], chars[i + 1]);
            bi_letters.push(bi_letter);
        }

        // get the vector of indices for the first bi-letter
        let mut indices = match self.index.get(&bi_letters[0]) {
            Some(indices) => indices.clone(),
            None => {
                return Vec::new(); // If the first bi-letter is not found, return an empty vector
            }
        };
        // Iterate over the remaining bi-letters and filter the indices
        for bi_letter in &bi_letters[1..] {
            if let Some(next_indices) = self.index.get(bi_letter) {
                // Only keep indices that are present in both the current indices and the next indices
                // As both lists are sorted, we can use a two-pointer technique
                let mut filtered_indices = Vec::new();
                let mut i = 0;
                let mut j = 0;
                while i < indices.len() && j < next_indices.len() {
                    if indices[i] == next_indices[j] {
                        filtered_indices.push(indices[i]);
                        i += 1;
                        j += 1;
                    } else if indices[i] < next_indices[j] {
                        i += 1; // Move to the next index in the current indices
                    } else {
                        j += 1; // Move to the next index in the next indices
                    }
                }
                indices = filtered_indices; // Update indices to the filtered list
            } else {
                // If no indices found for the current bi-letter, return empty results
                return Vec::new();
            }
        }

        indices
    }

    pub fn len(&self) -> usize {
        // Return the number of bi-letter entries in the index
        self.index.len()
    }
}

fn create_bi_letter_reverse_index(tree: &FileTree) -> HashMap<String, Vec<usize>> {
    // Create a bi-letter reverse index for the elements
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, element) in tree.get_elements().iter().enumerate() {
        if element.filename.len() < 2 {
            continue; // Skip filenames that are too short for bi-letter indexing
        }
        // take every two letters of the filename
        let filename = element.filename.to_lowercase();
        // be aware of unicode characters, we need to handle them properly
        let char_vec: Vec<char> = filename.chars().collect();
        for j in 0..char_vec.len() - 1 {
            // Create a bi-letter from the current and next character
            let bi_letter = format!("{}{}", char_vec[j], char_vec[j + 1]);
            // Insert the index of the element into the index map
            index.entry(bi_letter).or_default().push(i);
        }
    }

    // Ensure indices are unique and sorted
    for indices in index.values_mut() {
        indices.dedup(); // Remove duplicates
        indices.sort_unstable();
        indices.shrink_to_fit(); // Reduce capacity to the actual number of indices
    }
    index
}
