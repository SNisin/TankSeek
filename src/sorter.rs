use core::time;
use std::sync::Mutex;

use crate::file_tree::FileTree;

pub enum SortField {
    Filename,
    DateModified,
    DateCreated,
    Size,
}

pub enum SortOrder {
    Ascending,
    Descending,
}
pub struct Sorter {
    pub filename_order: Mutex<Option<Vec<usize>>>,
}
impl Sorter {
    pub fn new() -> Self {
        Sorter {
            filename_order: Mutex::new(None),
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
                self.sort_by_filename(tree, elements, order);
            }
            SortField::DateModified => {
                unimplemented!();
                // self.sort_by_date_modified(tree, elements, order);
            }
            SortField::DateCreated => {
                unimplemented!();
                // self.sort_by_date_created(tree, elements, order);
            }
            SortField::Size => {
                unimplemented!();
                // self.sort_by_size(tree, elements, order);
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

            println!("Filename order prepared with {} entries in {:?}", order.len(), timestamp.elapsed());
            filename_order.replace(order);
        }
    }
    fn sort_by_filename(&self, tree: &FileTree, elements: &mut [usize], order: SortOrder) {
        self.prepare_filename_order(tree);
        let filename_order_guard = self.filename_order.lock().unwrap();
        let filename_order = filename_order_guard.as_ref().unwrap();
        match order {
            SortOrder::Ascending => {
                elements.sort_unstable_by(|&a, &b| filename_order[a].cmp(&filename_order[b]));
            }
            SortOrder::Descending => {
                elements.sort_unstable_by(|&a, &b| filename_order[b].cmp(&filename_order[a]));
            }
        }
    }
}
