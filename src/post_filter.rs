// use std::time::Instant;

use crate::file_tree::FileTree;

pub fn post_filter(tree: &FileTree, indices: &mut Vec<usize>, query: &str) {
    // let start_time = Instant::now();
    // let original_len = indices.len();

    let regex = regex::RegexBuilder::new(&regex::escape(query))
        .case_insensitive(true)
        .build()
        .expect("Failed to compile regex");

    // Filter results based on the query

    indices.retain(|&index| regex.is_match(&tree.get_filename(index)));

    // print!(
    //     "Post-filtering took {} ms, reduced results from {} to {}\n",
    //     start_time.elapsed().as_millis(),
    //     original_len,
    //     indices.len()
    // );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_tree::{Element, FileTree};

    #[test]
    fn test_post_filter() {
        let mut tree = FileTree::with_capacity(5);
        let element1 = tree.add_element(Element {
            filename: tree.new_filename("file1.txt"),
            size: Some(1000),
            date_modified: Some(1000),
            date_created: Some(1000),
            attributes: 0,
            parent: 0,
            children: Vec::new(),
        });
        let element2 = tree.add_element(Element {
            filename: tree.new_filename("file2.txt"),
            size: Some(2000),
            date_modified: Some(2000),
            date_created: Some(2000),
            attributes: 0,
            parent: 0,
            children: Vec::new(),
        });
        let element3 = tree.add_element(Element {
            filename: tree.new_filename("file3.txt"),
            size: Some(3000),
            date_modified: Some(3000),
            date_created: Some(3000),
            attributes: 0,
            parent: 0,
            children: Vec::new(),
        });
        let element4 = tree.add_element(Element {
            filename: tree.new_filename("file4.txt"),
            size: Some(4000),
            date_modified: Some(4000),
            date_created: Some(4000),
            attributes: 0,
            parent: 0,
            children: Vec::new(),
        });
        let mut indices = vec![element1, element2, element3, element4];
        post_filter(&tree, &mut indices, "file2");
        assert_eq!(indices, vec![element2]);
        post_filter(&tree, &mut indices, "file3");
        assert!(indices.is_empty());
    }
}
