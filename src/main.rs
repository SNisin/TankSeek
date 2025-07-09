use rocket::http::ContentType;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    io,
    process::{self, exit},
};
#[derive(Deserialize, Serialize)]
struct Record {
    #[serde(rename = "Filename")]
    filename: String,
    #[serde(rename = "Size")]
    size: Option<i64>,
    #[serde(rename = "Date Modified")]
    date_modified: Option<i64>,
    #[serde(rename = "Date Created")]
    date_created: Option<i64>,
    #[serde(rename = "Attributes")]
    attributes: u32,
}

#[derive(Serialize, Clone)]
struct Element {
    filename: String,
    size: Option<i64>,
    date_modified: Option<i64>,
    date_created: Option<i64>,
    attributes: u32,
    #[serde(skip)]
    parent: usize,
    #[serde(skip)]
    children: Vec<usize>,
}
fn find_child(element: usize, name: &str, elements: &Vec<Element>) -> Option<usize> {
    // Find the child element with the given name under the specified parent element
    elements[element]
        .children
        .iter()
        .find(|&&child_index| elements[child_index].filename == name)
        .cloned()
}
fn add_child(parent: usize, mut child: Element, elements: &mut Vec<Element>) -> usize {
    // Add a child element to the specified parent element
    let child_index = elements.len();
    elements[parent].children.push(child_index);
    child.parent = parent;
    elements.push(child);
    child_index
}

fn read_file_list() -> Result<Vec<Element>, Box<dyn Error>> {
    let file_list_reader = std::fs::File::open("filelist.efu")?;

    // Estimate the number of records in the file
    let file_size = file_list_reader.metadata()?.len();
    // Assuming an average record size of 100 bytes, adjust as necessary
    let estimated_records = (file_size / 100) as usize;
    // List of elements to build the tree structure
    let mut elements: Vec<Element> = Vec::with_capacity(estimated_records);
    //root element
    elements.push(Element {
        filename: String::from("Root"),
        size: None,
        date_modified: None,
        date_created: None,
        attributes: 0,        // Assuming root has no attributes
        parent: 0,            // Root has no parent
        children: Vec::new(), // Root has no children initially
    });

    // Create a CSV reader from the file
    let mut rdr = csv::Reader::from_reader(file_list_reader);

    // Cache HashMap to store the elements by path
    let mut elements_map: HashMap<String, usize> = HashMap::new();

    // Iterate over the records and build the tree structure
    for record in rdr.deserialize() {
        let record: Record = record?;
        // Split the filename into parts \ and /
        let parts: Vec<&str> = record.filename.split(&['\\', '/'][..]).collect();
        let mut current_element = 0; // Start with the root element

        // find longest existing path in the elements_map try longest path first
        let mut start_part: i32 = 0;
        for i in (0..parts.len()).rev() {
            let path = parts[0..=i].join("/");
            if let Some(&index) = elements_map.get(&path) {
                current_element = index;
                start_part = (i as i32) + 1; // Store the index of the last part of the longest existing path
                break; // Found the longest existing path, no need to check further
            }
        }

        for (i, part) in parts.iter().enumerate().skip(start_part as usize) {
            // create a new child element
            let new_element = Element {
                filename: part.to_string(),
                size: None,
                date_modified: None,
                date_created: None,
                attributes: 0,
                parent: current_element, // Set the parent to the current element
                children: Vec::new(),    // New element has no children initially
            };
            current_element = add_child(current_element, new_element, &mut elements);

            // Update the elements_map with the new path
            let full_path = parts[0..=i].join("/");
            elements_map.insert(full_path, current_element);
        }

        // we need to update the existing element with the record data
        if let Some(existing_element) = elements.get_mut(current_element) {
            existing_element.size = record.size;
            existing_element.date_modified = record.date_modified;
            existing_element.date_created = record.date_created;
            existing_element.attributes = record.attributes;
        } else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                "Element not found for update",
            )));
        }

        // println!("Added file: {}", record.filename);
    }

    // Reduce capacity to the actual number of elements
    elements.shrink_to_fit();
    // Return the elements as a vector
    Ok(elements)
}

fn create_bi_letter_reverse_index(elements: &Vec<Element>) -> HashMap<String, Vec<usize>> {
    // Create a bi-letter reverse index for the elements
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, element) in elements.iter().enumerate() {
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

#[macro_use]
extern crate rocket;

// serve static index.html
#[get("/")]
fn index() -> (ContentType, &'static str) {
    (ContentType::HTML, include_str!("../public/index.html"))
}

#[get("/search?<query>")]
fn search(
    query: String,
    elements: &rocket::State<Vec<Element>>,
    bi_letter_index: &rocket::State<HashMap<String, Vec<usize>>>,
) -> String {
    let mut results = Vec::new();
    // Normalize the query to lowercase for case-insensitive search
    let query = query.to_lowercase();

    // Check if the query is empty
    if query.is_empty() {
        results = elements
            .iter()
            .take(100) // Limit to 100 results
            .cloned()
            .collect();
    } else if query.len() < 2 {
        // If the query is less than 2 characters, TODO
        return String::from("[]");
    } else {
        // Split the query into bi-letters
        let mut bi_letters = Vec::new();
        let chars: Vec<char> = query.chars().collect();
        for i in 0..chars.len() - 1 {
            // Create a bi-letter from the current and next character
            let bi_letter = format!("{}{}", chars[i], chars[i + 1]);
            bi_letters.push(bi_letter);
        }

        // get the vector of indices for the first bi-letter
        let mut indices = match bi_letter_index.get(&bi_letters[0]) {
            Some(indices) => indices.clone(),
            None => {
                return String::from("[]");
            }
        };
        // Iterate over the remaining bi-letters and filter the indices
        for bi_letter in &bi_letters[1..] {
            if let Some(next_indices) = bi_letter_index.get(bi_letter) {
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
                return String::from("[]");
            }
        }

        // Now we have the indices of the elements that match the query
        let mut num_results = 0; // Counter for the number of results
        // Prepare the results based on the indices
        for &index in &indices {
            if num_results < 100 {
                // If we have less than 100 results, add the record to the results
                let mut record_with_full_path = elements[index].clone();
                // Construct the full path for the record from parents
                let mut full_path = record_with_full_path.filename.clone();
                let mut parent_index = record_with_full_path.parent;
                while parent_index != 0 {
                    if let Some(parent) = elements.get(parent_index) {
                        full_path = format!("{}/{}", parent.filename, full_path);
                        parent_index = parent.parent;
                    } else {
                        break; // If parent not found, break the loop
                    }
                }
                // Update the filename to the full path
                record_with_full_path.filename = full_path;
                // Add the record to the results
                results.push(record_with_full_path);
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

    // Convert results to JSON
    match serde_json::to_string(&results) {
        Ok(json) => json,
        Err(e) => format!("Error serializing results: {}", e),
    }
}

#[launch]
fn rocket() -> _ {
    println!("Reading file list...");
    match read_file_list() {
        Ok(elements) => {
            println!("Read {} records from filelist.efu", elements.len());

            // Create a bi-letter reverse index for the elements
            println!("Creating bi-letter reverse index...");
            let bi_letter_index = create_bi_letter_reverse_index(&elements);
            println!(
                "Created bi-letter reverse index with {} entries",
                bi_letter_index.len()
            );

            //  exit(0); // Exit successfully after reading the file list
            rocket::build()
                .manage(elements)
                .manage(bi_letter_index)
                .mount("/", routes![index, search])
        }
        Err(e) => {
            eprintln!("Error reading file list: {}", e);
            process::exit(1);
        }
    }
}
