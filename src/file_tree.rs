// Filename struct to represent a filename with start index and end in byte array
pub struct Filename(usize, usize);
impl Filename {
    pub fn new(start: usize, end: usize) -> Self {
        Filename(start, end)
    }
    pub fn len(&self) -> usize {
        self.1 - self.0
    }
}

pub struct Element {
    pub filename: Filename,
    pub size: Option<i64>,
    pub date_modified: Option<i64>,
    pub date_created: Option<i64>,
    pub attributes: u32,
    pub parent: usize,
    pub children: Vec<usize>,
}
impl Element {}

pub struct FileTree {
    pub elements: Vec<Element>,
    strbuf: Vec<u8>, // Buffer for storing filenames as byte arrays
}
impl FileTree {
    pub fn with_capacity(capacity: usize) -> Self {
        // create a new FileTree with a specified initial capacity and a root element
        let mut tree = FileTree {
            elements: Vec::with_capacity(capacity),
            strbuf: Vec::with_capacity(capacity * 10), // Initial capacity for the string buffer
        };
        // Add a root element
        tree.add_root();
        tree
    }

    pub fn add_element(&mut self, element: Element) -> usize {
        let index = self.elements.len();
        self.elements.push(element);
        index
    }

    fn add_root(&mut self) -> usize {
        // Add a root element if it doesn't exist
        if self.elements.is_empty() {
            let filename = self.new_filename("Root");
            let root = Element {
                filename: filename,
                size: None,
                date_modified: None,
                date_created: None,
                attributes: 0,
                parent: 0, // Root has no parent
                children: Vec::new(),
            };
            self.add_element(root)
        } else {
            0 // Return the index of the existing root element
        }
    }

    pub fn add_or_update_recursive(
        &mut self,
        path: &str,
        size: Option<i64>,
        date_modified: Option<i64>,
        date_created: Option<i64>,
        attributes: u32,
    ) -> usize {
        let mut current_index = 0; // Start from the root
        for part in path.split(&['\\', '/']) {
            // println!("Part: {}, current_index: {}", part, current_index);
            // if part == "tank" { panic!("Debugging"); }

            // Check if the part already exists among the children
            let found_elem = self.elements[current_index]
                .children
                .binary_search_by_key(&part, |&child_index| self.get_filename(child_index));
            // println!("Found elem: {:?}", found_elem);
            current_index = match found_elem {
                Ok(index) => self.elements[current_index].children[index], // Move to the existing child
                Err(index) => {
                    // Create a new element
                    let new_element = Element {
                        filename: self.new_filename(part),
                        size: None,
                        date_modified: None,
                        date_created: None,
                        attributes: 0,
                        parent: current_index,
                        children: Vec::new(),
                    };
                    let child_index = self.add_element(new_element);
                    self.elements[current_index]
                        .children
                        .insert(index, child_index);
                    child_index
                }
            };
        }
        // Update the final element with the provided metadata
        if let Some(element) = self.elements.get_mut(current_index) {
            element.size = size;
            element.date_modified = date_modified;
            element.date_created = date_created;
            element.attributes = attributes;
        }
        current_index
    }

    pub fn new_filename(&mut self, string: &str) -> Filename {
        // Create a new Filename from a string, storing it in the strbuf
        let start = self.strbuf.len();
        self.strbuf.extend_from_slice(string.as_bytes());
        let end = self.strbuf.len();
        Filename::new(start, end)
    }

    pub fn get(&self, index: usize) -> Option<&Element> {
        self.elements.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Element> {
        self.elements.get_mut(index)
    }
    pub fn get_elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn filename_as_str(&self, filename: &Filename) -> &str {
        // Convert the byte slice to a str using the start and end indices
        let filename_bytes = &self.strbuf[filename.0..filename.1];
        // Convert bytes to str, assuming UTF-8 encoding
        std::str::from_utf8(filename_bytes).unwrap_or("")
    }
    pub fn get_filename(&self, index: usize) -> &str {
        // Get the filename of the element at the specified index
        let element = &self.elements[index];
        self.filename_as_str(&element.filename)
    }

    pub fn get_full_path(&self, index: usize) -> String {
        // Get the path of the element at the specified index. Not including the filename itself.
        let mut path = String::new();
        let mut current_index = index;
        while current_index != 0 {
            let element = &self.elements[current_index];
            if !path.is_empty() {
                path = format!("{}\\{}", self.filename_as_str(&element.filename), path);
            } else {
                path = self.filename_as_str(&element.filename).to_string();
            }
            current_index = element.parent;
        }
        path
    }

    pub fn collect_all_children(&self, index: usize) -> Vec<usize> {
        // Collect all children of the specified element recursively
        let mut children = Vec::new();
        if let Some(element) = self.get(index) {
            for &child_index in &element.children {
                children.push(child_index);
                children.extend(self.collect_all_children(child_index));
            }
        }
        children
    }

    pub fn add_child(&mut self, parent: usize, mut child: Element) -> usize {
        // Add a child element to the specified parent element
        let child_index = self.elements.len();
        self.elements[parent].children.push(child_index);
        child.parent = parent;
        self.elements.push(child);
        child_index
    }
    pub fn shrink_to_fit(&mut self) {
        // Reduce the capacity of the elements vector to fit the current number of elements
        self.elements.shrink_to_fit();
    }
    pub fn len(&self) -> usize {
        // Return the number of elements in the tree
        self.elements.len()
    }
}
