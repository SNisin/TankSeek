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
    use crate::file_tree::FileTree;

    #[test]
    fn test_post_filter() {
        let mut tree = FileTree::with_capacity(5);
        let element1 = tree.add_or_update_recursive(
            "some/path/file1.txt",
            Some(1000),
            Some(4000),
            Some(3000),
            0,
        );
        let element2 = tree.add_or_update_recursive(
            "other/path/file2.txt",
            Some(3000),
            Some(1000),
            Some(4000),
            0,
        );
        let element3 =
            tree.add_or_update_recursive("mydir/file3.txt", Some(2000), Some(2000), Some(2000), 0);
        let element4 =
            tree.add_or_update_recursive("C:/file4.txt", Some(4000), Some(3000), Some(1000), 0);
        let mut indices = vec![element1, element2, element3, element4];
        post_filter(&tree, &mut indices, "file2");
        assert_eq!(indices, vec![element2]);
        post_filter(&tree, &mut indices, "file3");
        assert!(indices.is_empty());
    }
}
