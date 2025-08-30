use rocket::fs::{FileServer, relative};
use serde::{Deserialize, Serialize};
use std::process::{self};
use std::sync::Mutex;
mod file_tree;
mod indexer;
mod loader;
mod post_filter;
mod sorter;
use crate::searcher::Searcher;
use crate::sorter::{SortField, SortOrder};
use std::time::Instant;
mod searcher;

#[derive(Serialize, Deserialize, Clone)]
struct FileResult {
    name: String,
    path: String,
    size: Option<i64>,
    date_modified: Option<i64>,
    date_created: Option<i64>,
    attributes: u32,
}
impl FileResult {
    fn from_element<T: AsRef<str>, U: AsRef<str>>(element: &file_tree::Element, path: T, filename: U) -> Self {
        FileResult {
            name: filename.as_ref().to_string(),
            path: path.as_ref().to_string(),
            size: element.size,
            date_modified: element.date_modified,
            date_created: element.date_created,
            attributes: element.attributes,
        }
    }
}
#[derive(Serialize, Deserialize)]
struct SearchResult {
    results: Vec<FileResult>,
    total: usize,
    offset: usize,
    page_size: usize,
    time_taken: u128,
}

struct SearchCache {
    query: String,
    indices: Vec<usize>,
    sort_by: Option<SortField>,
    sort_order: Option<SortOrder>,
}
struct LastSearchCache {
    search: Mutex<Option<SearchCache>>,
}

#[macro_use]
extern crate rocket;

#[get("/search?<query>&<offset>&<sort_by>&<sort_order>")]
fn search(
    query: String,
    offset: Option<usize>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    searcher: &rocket::State<Searcher>,
    last_search_cache: &rocket::State<LastSearchCache>,
) -> String {
    let time_start = Instant::now();
    let result_indices;

    // Normalize the query to lowercase for case-insensitive search
    let query = query.to_lowercase();

    let sort_by: Option<SortField> = match sort_by.as_deref() {
        Some("filename") => Some(SortField::Filename),
        Some("date_modified") => Some(SortField::DateModified),
        Some("date_created") => Some(SortField::DateCreated),
        Some("size") => Some(SortField::Size),
        _ => None, // Default to None if no valid sort field is provided
    };
    let sort_order: Option<SortOrder> = match sort_order.as_deref() {
        Some("ascending") => Some(SortOrder::Ascending),
        Some("descending") => Some(SortOrder::Descending),
        _ => None, // Default to None if no valid sort order is provided
    };

    // Check if the query is cached
    let mut cache_guard = last_search_cache.search.lock().unwrap();
    if let Some(cache) = cache_guard.as_ref()
        && cache.query == query
        && cache.sort_by == sort_by
        && cache.sort_order == sort_order
    {
        result_indices = &cache.indices;
    } else {
        drop(cache_guard); // Release the lock before performing the search

        // Perform the search using the Searcher
        let indices = searcher.search(&query, sort_by, sort_order);

        cache_guard = last_search_cache.search.lock().unwrap();
        cache_guard.replace(SearchCache {
            query: query.clone(),
            indices: indices,
            sort_by,
            sort_order,
        });
        result_indices = &cache_guard.as_ref().unwrap().indices;
    }
    let mut result_elements = Vec::new();
    // Now we have the indices of the elements that match the query
    // Prepare the results based on the indices
    result_indices
        .iter()
        .skip(offset.unwrap_or(0))
        .take(100)
        .for_each(|&index| {
            if let Some(element) = searcher.get(index) {
                result_elements.push(element);
            }
        });

    // Convert the elements to FileResult
    let results: Vec<_> = result_elements
        .into_iter()
        .map(|element| {
            FileResult::from_element(
                &element,
                searcher.get_file_tree().get_full_path(element.parent),
                searcher.get_file_tree().filename_as_str(&element.filename)
            )
        })
        .collect();

    let results = SearchResult {
        results,
        total: result_indices.len(),
        offset: offset.unwrap_or(0),
        page_size: 100, // Fixed page size for now
        time_taken: time_start.elapsed().as_micros(),
    };
    // Convert results to JSON
    match serde_json::to_string(&results) {
        Ok(json) => json,
        Err(e) => format!("Error serializing results: {}", e),
    }
}

#[launch]
fn rocket() -> _ {
    println!("Reading file list...");
    let start = Instant::now();
    match loader::efu::import_efu("filelist.efu") {
        Ok(tree) => {
            println!(
                "Read {} records from filelist.efu in {:?}",
                tree.len(),
                start.elapsed()
            );

            // Create searcher
            let searcher = Searcher::from_file_tree(tree);

            //  exit(0); // Exit successfully after reading the file list
            rocket::build()
                .manage(searcher)
                .manage(LastSearchCache {
                    search: Mutex::new(None),
                })
                .mount("/", routes![search])
                .mount("/", FileServer::from(relative!("public")))
        }
        Err(e) => {
            eprintln!("Error reading file list: {}", e);
            process::exit(1);
        }
    }
}
