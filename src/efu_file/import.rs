use std::{collections::HashMap, error::Error, io, path::Path};

use serde::{Deserialize, Serialize};

use bumpalo::{Bump, collections::Vec as BumpVec};

use crate::file_tree::{Element, FileTree};

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

pub fn import_efu<P: AsRef<Path>>(filepath: P) -> Result<FileTree, Box<dyn Error>> {
    let file_list_reader = std::fs::File::open(filepath)?;

    // Estimate the number of records in the file
    let file_size = file_list_reader.metadata()?.len();
    // Assuming an average record size of 100 bytes, adjust as necessary
    let estimated_records = (file_size / 100) as usize;
    // List of elements to build the tree structure
    let mut tree: FileTree = FileTree::with_capacity(estimated_records);

    // Create a CSV reader from the file
    let mut rdr = csv::Reader::from_reader(file_list_reader);

    // Cache HashMap to store the elements by path
    let mut elements_map: HashMap<String, usize> = HashMap::new();

    let mut bump = Bump::with_capacity(1000);

    // Iterate over the records and build the tree structure
    for record in rdr.deserialize() {
        let record: Record = record?;
        // Split the filename into parts \ and /
        let mut parts: BumpVec<&str> = BumpVec::new_in(&bump);
        parts.extend(record.filename.split(&['\\', '/'][..]));
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
            current_element = tree.add_child(current_element, new_element);

            // Update the elements_map with the new path
            let full_path = parts[0..=i].join("/");
            elements_map.insert(full_path, current_element);
        }

        // we need to update the existing element with the record data
        if let Some(existing_element) = tree.get_mut(current_element) {
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

        drop(parts);
        bump.reset(); // Reset the bump allocator for the next iteration

        // println!("Added file: {}", record.filename);
    }

    // Reduce capacity to the actual number of elements
    tree.shrink_to_fit();
    // Return the elements as a vector
    Ok(tree)
}
