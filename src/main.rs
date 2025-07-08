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

    // Convert the records to elements
    let mut elements: Vec<Element> = Vec::new();
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
                let mut record_with_full_path = record.clone();
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
