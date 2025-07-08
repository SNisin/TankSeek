use rocket::http::{ContentType, uri::fmt::Part};
use serde::{Deserialize, Serialize};
use std::{error::Error, io, process::{self, exit}};
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

#[derive(Serialize)]
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
    let mut file_records: Vec<Record> = Vec::new();
    let file_list_reader = std::fs::File::open("filelist.efu")?;
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(file_list_reader);
    for record in rdr.deserialize() {
        file_records.push(record?);
    }

    // Convert the records to elements
    let mut elements: Vec<Element> = Vec::new();
    elements.reserve(file_records.len() + 1); // Reserve space for elements, +1 for root
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

    // Iterate over the records and build the tree structure
    for record in file_records {
        // Split the filename into parts \ and /
        let parts: Vec<&str> = record.filename.split(&['\\', '/'][..]).collect();
        let mut current_element = 0; // Start with the root element
        let mut last_created = false; // Flag to check if we created the last element

        for part in parts {
            // Check if the part already exists as a child of the current element
            if let Some(child_index) = find_child(current_element, part, &elements) {
                // If it exists, move to that child
                current_element = child_index;
                last_created = false;
            } else {
                // If it doesn't exist, create a new child element
                let new_element = Element {
                    filename: part.to_string(),
                    size: record.size,
                    date_modified: record.date_modified,
                    date_created: record.date_created,
                    attributes: record.attributes,
                    parent: current_element, // Set the parent to the current element
                    children: Vec::new(),    // New element has no children initially
                };
                current_element = add_child(current_element, new_element, &mut elements);
                last_created = true; // We created a new element
            }
        }
        // For last part, we need to update the size, date_modified, date_created, and attributes
        if !last_created {
            // If we didn't create a new element, we need to update the existing one
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
        }
        // println!("Added file: {}", record.filename);
    }
    // Return the elements as a vector
    Ok(elements)
}

#[macro_use]
extern crate rocket;

// serve static index.html
#[get("/")]
fn index() -> (ContentType, &'static str) {
    (ContentType::HTML, include_str!("../public/index.html"))
}

#[get("/search?<query>")]
fn search(query: String, elements: &rocket::State<Vec<Element>>) -> String {
    let mut results = Vec::new();
    // Normalize the query to lowercase for case-insensitive search
    let query = query.to_lowercase();
    // Iterate over the records and filter based on the query
    // limit to 100 results but count all matching records
    let mut num_results = 0;
    for record in elements.iter() {
        if record.filename.to_lowercase().contains(&query) {
            if num_results < 100 {
                // If we have less than 100 results, add the record to the results
                results.push(record);
            } else {
                break;
            }
            // If we have 100 results, we still count the record but don't add it to the results
            num_results += 1;
        }
    }
    println!(
        "Found {} matching records for query '{}'",
        num_results, query
    );
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

            // exit(0); // Exit successfully after reading the file list
            rocket::build()
                .manage(elements)
                .mount("/", routes![index, search])
        }
        Err(e) => {
            eprintln!("Error reading file list: {}", e);
            process::exit(1);
        }
    }
}
