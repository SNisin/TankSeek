use std::{error::Error, path::Path};

use serde::{Deserialize, Serialize};

use crate::file_tree::FileTree;

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

    // Iterate over the records and build the tree structure
    for record in rdr.deserialize() {
        let record: Record = record?;
        tree.add_or_update_recursive(
            &record.filename,
            record.size,
            record.date_modified,
            record.date_created,
            record.attributes,
        );

        // println!("Added file: {}", record.filename);
    }

    // Reduce capacity to the actual number of elements
    tree.shrink_to_fit();
    // Return the elements as a vector
    Ok(tree)
}
