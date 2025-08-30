use crate::{
    file_tree::{self, FileTree},
    indexer::bigram_index::BigramIndex,
    post_filter,
    sorter::{SortField, SortOrder, Sorter},
};

pub struct Searcher {
    pub file_tree: FileTree,
    pub bigram_index: BigramIndex,
    pub sorter: Sorter,
}

impl Searcher {
    pub fn from_file_tree(tree: FileTree) -> Self {
        let bigram_index = BigramIndex::new(&tree);
        let sorter = Sorter::new();
        Searcher {
            file_tree: tree,
            bigram_index,
            sorter,
        }
    }

    pub fn search<T: AsRef<str>>(
        &self,
        query: T,
        sort_by: Option<SortField>,
        sort_order: Option<SortOrder>,
    ) -> Vec<usize> {
        let mut indices: Vec<usize>;

        // Normalize the query to lowercase for case-insensitive search
        let query = query.as_ref().to_lowercase();
        let query_len = query.chars().count();

        // Search
        if query.is_empty() {
            // query is empty, return all indices
            indices = (0..self.file_tree.len()).collect::<Vec<usize>>();
        } else if query_len < 2 {
            // query is 1 character
            indices = self.bigram_index.query_char(query.chars().next().unwrap());
        } else {
            // query is longer than 1 character
            indices = self.bigram_index.query_word(&query);
            if query_len > 2 {
                // If the query is longer than 2 characters, apply post-filtering
                post_filter::post_filter(&self.file_tree, &mut indices, &query);
            }
        }

        println!(
            "Found {} matching records for query '{}'",
            indices.len(),
            query
        );
        // Sort results if a sort field is provided
        if let Some(sort_by) = sort_by {
            let sort_order = sort_order.unwrap_or(SortOrder::Ascending);
            self.sorter
                .sort_by(&self.file_tree, indices.as_mut_slice(), sort_by, sort_order);
        }
        indices
    }

    pub fn get_file_tree(&self) -> &FileTree {
        &self.file_tree
    }
    pub fn get(&self, index: usize) -> Option<&file_tree::Element> {
        self.file_tree.get(index)
    }
}
