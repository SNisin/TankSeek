use crate::file_tree::FileTree;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Filename,
    DateModified,
    DateCreated,
    Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}
pub struct Sorter {
    pub filename_order: Mutex<Option<Vec<usize>>>,
    pub date_modified_order: Mutex<Option<Vec<usize>>>,
    pub date_created_order: Mutex<Option<Vec<usize>>>,
    pub size_order: Mutex<Option<Vec<usize>>>,
}
impl Sorter {
    pub fn new() -> Self {
        Sorter {
            filename_order: Mutex::new(None),
            date_modified_order: Mutex::new(None),
            date_created_order: Mutex::new(None),
            size_order: Mutex::new(None),
        }
    }

    pub fn sort_by(
        &self,
        tree: &FileTree,
        elements: &mut [usize],
        field: SortField,
        order: SortOrder,
    ) {
        match field {
            SortField::Filename => {
                self.prepare_filename_order(tree);
                let filename_order = self.filename_order.lock().unwrap();
                self.sort_by_order_list(elements, filename_order.as_ref().unwrap(), order);
            }
            SortField::DateModified => {
                self.prepare_date_modified_order(tree);
                let date_modified_order = self.date_modified_order.lock().unwrap();
                self.sort_by_order_list(elements, date_modified_order.as_ref().unwrap(), order);
            }
            SortField::DateCreated => {
                self.prepare_date_created_order(tree); // Reuse the same method for date created
                let date_created_order = self.date_created_order.lock().unwrap();
                self.sort_by_order_list(elements, date_created_order.as_ref().unwrap(), order);
            }
            SortField::Size => {
                self.prepare_size_order(tree);
                let size_order = self.size_order.lock().unwrap();
                self.sort_by_order_list(elements, size_order.as_ref().unwrap(), order);
            }
        }
    }
    fn prepare_filename_order(&self, tree: &FileTree) {
        let mut filename_order = self.filename_order.lock().unwrap();
        if filename_order.is_none() {
            println!("Preparing filename order...");
            let timestamp = std::time::Instant::now();
            let mut sorted: Vec<usize> = (0..tree.get_elements().len()).collect();
            sorted.sort_by(|&a, &b| {
                tree.get(a)
                    .unwrap()
                    .filename
                    .cmp(&tree.get(b).unwrap().filename)
            });
            let mut order = vec![0; sorted.len()];

            for (i, &index) in sorted.iter().enumerate() {
                order[index] = i;
            }

            println!(
                "Filename order prepared with {} entries in {:?}",
                order.len(),
                timestamp.elapsed()
            );
            filename_order.replace(order);
        }
    }

    fn prepare_date_modified_order(&self, tree: &FileTree) {
        let mut date_modified_order = self.date_modified_order.lock().unwrap();
        if date_modified_order.is_none() {
            println!("Preparing date modified order...");
            let timestamp = std::time::Instant::now();
            let mut sorted: Vec<usize> = (0..tree.get_elements().len()).collect();
            sorted.sort_by(|&a, &b| {
                tree.get(a)
                    .unwrap()
                    .date_modified
                    .cmp(&tree.get(b).unwrap().date_modified)
            });
            let mut order = vec![0; sorted.len()];

            for (i, &index) in sorted.iter().enumerate() {
                order[index] = i;
            }

            println!(
                "Date modified order prepared with {} entries in {:?}",
                order.len(),
                timestamp.elapsed()
            );
            date_modified_order.replace(order);
        }
    }

    fn prepare_date_created_order(&self, tree: &FileTree) {
        let mut date_created_order = self.date_created_order.lock().unwrap();
        if date_created_order.is_none() {
            println!("Preparing date created order...");
            let timestamp = std::time::Instant::now();
            let mut sorted: Vec<usize> = (0..tree.get_elements().len()).collect();
            sorted.sort_by(|&a, &b| {
                tree.get(a)
                    .unwrap()
                    .date_created
                    .cmp(&tree.get(b).unwrap().date_created)
            });
            let mut order = vec![0; sorted.len()];

            for (i, &index) in sorted.iter().enumerate() {
                order[index] = i;
            }

            println!(
                "Date created order prepared with {} entries in {:?}",
                order.len(),
                timestamp.elapsed()
            );
            date_created_order.replace(order);
        }
    }

    fn prepare_size_order(&self, tree: &FileTree) {
        let mut size_order = self.size_order.lock().unwrap();
        if size_order.is_none() {
            println!("Preparing size order...");
            let timestamp = std::time::Instant::now();
            let mut sorted: Vec<usize> = (0..tree.get_elements().len()).collect();
            sorted.sort_by(|&a, &b| tree.get(a).unwrap().size.cmp(&tree.get(b).unwrap().size));
            let mut order = vec![0; sorted.len()];

            for (i, &index) in sorted.iter().enumerate() {
                order[index] = i;
            }

            println!(
                "Size order prepared with {} entries in {:?}",
                order.len(),
                timestamp.elapsed()
            );
            size_order.replace(order);
        }
    }

    fn sort_by_order_list(
        &self,
        elements: &mut [usize],
        order_list: &Vec<usize>,
        order: SortOrder,
    ) {
        let len = order_list.len();
        let mut elements_sorted: Vec<usize> = vec![usize::MAX; len];
        if order == SortOrder::Ascending {
            for &index in elements.as_ref() {
                elements_sorted[order_list[index]] = index;
            }
        } else {
            for &index in elements.as_ref() {
                // For descending order, we need to reverse the order
                elements_sorted[len - 1 - order_list[index]] = index;
            }
        }
        let mut counter = 0;
        for i in 0..elements_sorted.len() {
            if elements_sorted[i] != usize::MAX {
                elements[counter] = elements_sorted[i];
                counter += 1;
            }
        }
    }
}
