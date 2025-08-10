// test performance

fn main() {
    let start_time = std::time::Instant::now();
    println!("Loading file tree...");
    let tree = efu_search_webui_rust::loader::efu::import_efu("filelist.efu")
        .expect("Failed to load file tree");
    println!(
        "Loaded {} elements in {:?}",
        tree.len(),
        start_time.elapsed()
    );
    let searcher = efu_search_webui_rust::searcher::Searcher::from_file_tree(tree);
    let query = "Brand";
    let sort_by = Some(efu_search_webui_rust::sorter::SortField::Filename);
    let sort_order = Some(efu_search_webui_rust::sorter::SortOrder::Ascending);

    let start_time = std::time::Instant::now();
    let result = searcher.search(query, sort_by, sort_order);
    println!("Search took {} ms", start_time.elapsed().as_millis());
    println!("Found {} results for query '{}'", result.len(), query);
}
