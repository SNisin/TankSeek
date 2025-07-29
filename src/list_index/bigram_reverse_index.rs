use std::collections::HashMap;

use crate::file_tree::FileTree;

#[derive(Hash, Eq, PartialEq, Debug, Clone, PartialOrd, Ord)]
pub struct Bigram {
    pub first: char,
    pub second: char,
}

pub struct CompressedPostingsList {
    pub indices: Vec<u8>,
}
// This struct is used to store the compressed postings list
// It stores the gaps between the indices
// Uses variable byte codes https://nlp.stanford.edu/IR-book/html/htmledition/variable-byte-codes-1.html
impl CompressedPostingsList {
    pub fn new(postings_list: Vec<usize>) -> Self {
        let mut compressed_list = Vec::with_capacity(postings_list.len() * 2);
        let mut last_i = 0;
        let mut bytes: [u8; 10] = [0; 10]; // Buffer for variable byte encoding
        let mut bytes_index = 0; // Index for the bytes buffer
        for &i in &postings_list {
            let gap = i as usize - last_i as usize; // Calculate the gap
            last_i = i; // Update the last index

            // Encode the gap using variable byte encoding
            let mut value = gap;

            while value >= 128 {
                bytes[bytes_index] = (value & 0x7F) as u8; // Store the last 7 bits
                value >>= 7; // Shift right by 7 bits
                bytes_index += 1; // Move to the next byte
            }
            bytes[bytes_index] = (value & 0x7F) as u8; // Store the last byte
            bytes[0] |= 0x80; // Set the continuation bit for the last byte (when reverse)

            // Reverse the bytes
            for j in (0..=bytes_index).rev() {
                compressed_list.push(bytes[j]);
            }
            bytes_index = 0; // Reset the index for the next gap
        }
        compressed_list.shrink_to_fit(); // Reduce capacity to the actual size
        CompressedPostingsList {
            indices: compressed_list,
        }
    }
    pub fn decompress(&self) -> Vec<usize> {
        let mut postings_list = Vec::with_capacity(self.indices.len());
        let mut last_value = 0; // Last value to calculate gaps
        let mut current_value = 0;
        for &byte in &self.indices {
            if byte < 128 {
                current_value = (current_value << 7) | (byte as usize); // Add the byte to the current value
            } else {
                current_value = (current_value << 7) | (byte & 0x7F) as usize; // Add the byte without the continuation bit
                last_value += current_value; // Update the last value
                postings_list.push(last_value); // Add the decompressed index
                current_value = 0; // Reset for the next value
            }
        }
        postings_list.shrink_to_fit(); // Reduce capacity to the actual size
        postings_list
    }
}

pub struct BigramIndex {
    pub index: HashMap<Bigram, CompressedPostingsList>,
}
impl BigramIndex {
    pub fn new(tree: &FileTree) -> Self {
        let index = create_bigram_reverse_index(tree);
        BigramIndex { index }
    }

    pub fn query_word<T: AsRef<str>>(&self, word: T) -> Vec<usize> {
        // Split the query into bigrams (bi-letters)
        let mut bigrams = Vec::new();
        let chars: Vec<char> = word.as_ref().chars().collect();
        for i in 0..chars.len() - 1 {
            // Create a bigram from the current and next character
            let bigram = Bigram {
                first: chars[i],
                second: chars[i + 1],
            };
            bigrams.push(bigram);
        }

        bigrams.dedup();

        // get the vector of indices for the first bigram
        let mut indices = match self.index.get(&bigrams[0]) {
            Some(indices) => indices.decompress(),
            None => {
                return Vec::new(); // If the first bigram is not found, return an empty vector
            }
        };
        // Iterate over the remaining bigrams and filter the indices
        let mut filtered_indices = Vec::with_capacity(indices.len());
        for bigram in &bigrams[1..] {
            if let Some(postings_list) = self.index.get(bigram) {
                let next_indices = postings_list.decompress();
                // Only keep indices that are present in both the current indices and the next indices
                // As both lists are sorted, we can use a two-pointer technique
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

                (indices, filtered_indices) = (filtered_indices, indices); // Update indices to the filtered list
                filtered_indices.clear(); // Clear the filtered indices for the next iteration
            } else {
                // If no indices found for the current bigram, return empty results
                return Vec::new();
            }
        }
        indices.shrink_to_fit(); // Reduce capacity to the actual size
        indices
    }

    pub fn len(&self) -> usize {
        // Return size of the index
        self.index.len()
    }
}

fn create_bigram_reverse_index(tree: &FileTree) -> HashMap<Bigram, CompressedPostingsList> {
    // Create a bigram reverse index for the elements
    let mut index: HashMap<Bigram, Vec<usize>> = HashMap::new();
    for (i, element) in tree.get_elements().iter().enumerate() {
        if element.filename.len() < 2 {
            continue; // Skip filenames that are too short for bigram indexing
        }
        // take every two letters of the filename
        let filename = element.filename.to_lowercase();
        // Split the query into bigrams (bi-letters)
        let chars: Vec<char> = filename.chars().collect();
        for j in 0..chars.len() - 1 {
            // Create a bigram from the current and next character
            let bigram = Bigram {
                first: chars[j],
                second: chars[j + 1],
            };
            index.entry(bigram).or_default().push(i);
        }
    }
    // Ensure indices are unique and sorted
    for indices in index.values_mut() {
        indices.sort_unstable(); // Should already be sorted, but just in case
        indices.dedup(); // Remove duplicates
        indices.shrink_to_fit(); // Reduce capacity to the actual number of indices
    }

    let mut compressed_index: HashMap<Bigram, CompressedPostingsList> = HashMap::new();
    for (bigram, indices) in index {
        compressed_index.insert(bigram, CompressedPostingsList::new(indices));
    }

    compressed_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressed_postings_list() {
        let postings_list_tests = vec![
            vec![1, 2, 3, 4, 5],
            vec![100, 200, 300, 400],
            vec![1, 42357, 845376, 845378, 1047637],
            vec![
                142357,
                1844674407370955160,
                1844674407370955161,
                18446744073709551600,
                18446744073709551615,
            ],
        ];
        for postings_list in postings_list_tests {
            let compressed = CompressedPostingsList::new(postings_list.clone());
            println!("Compressed: {:?}", compressed.indices);
            let decompressed = compressed.decompress();
            assert_eq!(postings_list, decompressed);
        }
    }
}
