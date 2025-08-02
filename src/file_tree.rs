use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Element {
    pub filename: String,
    pub size: Option<i64>,
    pub date_modified: Option<i64>,
    pub date_created: Option<i64>,
    pub attributes: u32,
    #[serde(skip)]
    pub parent: usize,
    #[serde(skip)]
    pub children: Vec<usize>,
}
impl Element {
    fn new_root() -> Self {
        // Create a new root element
        Element {
            filename: String::from("Root"),
            size: None,
            date_modified: None,
            date_created: None,
            attributes: 0,        // Assuming root has no attributes
            parent: 0,            // Root has no parent
            children: Vec::new(), // Root has no children initially
        }
    }
}

pub struct FileTree {
    pub elements: Vec<Element>,
}
impl FileTree {
    pub fn with_capacity(capacity: usize) -> Self {
        // create a new FileTree with a specified initial capacity and a root element
        let mut tree = FileTree {
            elements: Vec::with_capacity(capacity),
        };
        // Add a root element
        tree.add_element(Element::new_root());
        tree
    }

    pub fn add_element(&mut self, element: Element) -> usize {
        let index = self.elements.len();
        self.elements.push(element);
        index
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

    pub fn get_full_path(&self, index: usize) -> String {
        // Get the path of the element at the specified index. Not including the filename itself.
        let mut path = String::new();
        let mut current_index = index;
        while current_index != 0 {
            let element = &self.elements[current_index];
            if !path.is_empty() {
                path = format!("{}\\{}", element.filename, path);
            } else {
                path = element.filename.clone();
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
