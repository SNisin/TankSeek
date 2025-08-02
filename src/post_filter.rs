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

    indices.retain(|&index| regex.is_match(&tree.elements[index].filename));

    // print!(
    //     "Post-filtering took {} ms, reduced results from {} to {}\n",
    //     start_time.elapsed().as_millis(),
    //     original_len,
    //     indices.len()
    // );
}
