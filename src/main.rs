use rocket::fs::{FileServer, relative};
use serde::{Deserialize, Serialize};
use std::process::{self};
mod efu_file;
mod file_tree;
mod list_index;
use crate::file_tree::{Element, FileTree};
use crate::{efu_file::import, list_index::bigram_reverse_index::BigramIndex};
use std::time::Instant;

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
    fn from_element<T: AsRef<str>>(element: &file_tree::Element, path: T) -> Self {
        FileResult {
            name: element.filename.clone(),
            path: path.as_ref().to_string(),
            size: element.size,
            date_modified: element.date_modified,
            date_created: element.date_created,
            attributes: element.attributes,
        }
    }
}

#[macro_use]
extern crate rocket;

#[get("/search?<query>")]
fn search(
    query: String,
    tree: &rocket::State<FileTree>,
    bigram_index: &rocket::State<BigramIndex>,
) -> String {
    let mut result_elements = Vec::new();
    // Normalize the query to lowercase for case-insensitive search
    let query = query.to_lowercase();

    // Check if the query is empty
    if query.is_empty() {
        result_elements = tree
            .get_elements()
            .iter()
            .take(100) // Limit to 100 results
            .collect();
    } else if query.len() < 2 {
        // If the query is less than 2 characters, TODO
        return String::from("[]");
    } else {
        let indices = bigram_index.query_word(&query);

        // Now we have the indices of the elements that match the query
        let mut num_results = 0; // Counter for the number of results
        // Prepare the results based on the indices
        for &index in &indices {
            if num_results < 100 {
                // If we have less than 100 results, add the record to the results
                result_elements.push(&tree.get_elements()[index]);
                num_results += 1; // Increment the number of results found
            } else {
                // If we have 100 results, we stop adding more
                break;
            }
        }
        println!(
            "Found {} matching records for query '{}'",
            num_results, query
        );
    }

    // Iterate over the records and filter based on the query
    // limit to 100 results but count all matching records

    // let mut num_results = 0;
    // for record in elements.iter() {
    //     if record.filename.to_lowercase().contains(&query) {
    //         if num_results < 100 {
    //             // If we have less than 100 results, add the record to the results
    //             let mut record_with_full_path = record.clone();
    //             // Construct the full path for the record from parents
    //             let mut full_path = record_with_full_path.filename.clone();
    //             let mut parent_index = record_with_full_path.parent;
    //             while parent_index != 0 {
    //                 if let Some(parent) = elements.get(parent_index) {
    //                     full_path = format!("{}/{}", parent.filename, full_path);
    //                     parent_index = parent.parent;
    //                 } else {
    //                     break; // If parent not found, break the loop
    //                 }
    //             }
    //             // Update the filename to the full path
    //             record_with_full_path.filename = full_path;
    //             // Add the record to the results
    //             results.push(record_with_full_path);
    //         } else {
    //             break;
    //         }
    //         // If we have 100 results, we still count the record but don't add it to the results
    //         num_results += 1;
    //     }
    // }

    // Convert the elements to FileResult
    let results: Vec<_> = result_elements
        .into_iter()
        .map(|element| FileResult::from_element(&element, tree.get_full_path(element.parent)))
        .collect();

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
    match import::import_efu("filelist.efu") {
        Ok(tree) => {
            println!(
                "Read {} records from filelist.efu in {:?}",
                tree.len(),
                start.elapsed()
            );

            // Create a bigram reverse index for the elements
            println!("Creating bigram reverse index...");
            let start = Instant::now();
            let bigram_index = BigramIndex::new(&tree);
            println!(
                "Created bigram reverse index with {} entries in {:?}",
                bigram_index.len(),
                start.elapsed()
            );

            //  exit(0); // Exit successfully after reading the file list
            rocket::build()
                .manage(tree)
                .manage(bigram_index)
                .mount("/", routes![search])
                .mount("/", FileServer::from(relative!("public")))
        }
        Err(e) => {
            eprintln!("Error reading file list: {}", e);
            process::exit(1);
        }
    }
}
