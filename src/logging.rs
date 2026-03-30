use std::{fs::{self, File}, path::Path};
use std::io::Write;


pub struct Logger {
    entry: Option<Entry>
}

pub struct Entry {
    scales:  File,
    task:    File,
    service: File,
}

impl Logger {
    pub fn new() -> Self {
        Self { entry: None }
    }

    pub fn set_order(&mut self, order_id: Option<i32>) {
        let Some(order_id) = order_id else {
            // If None, clear entry and return
            self.entry = None;
            return;
        };

        let folder_path = format!("/home/qitech/ff01_orders/order_{}", order_id);
        let path = Path::new(&folder_path);

        if path.exists() {
            fs::remove_dir_all(&path).expect("Failed to remove existing folder");
        }

        // Recursive directory creation
        fs::create_dir_all(&path).expect("Failed to create folder recursively");

        // Create 3 CSV files inside the folder
        let scales_path  = path.join("scales.csv");
        let task_path    = path.join("task.csv");
        let service_path = path.join("service.csv");

        let scales_file  = File::create(scales_path).expect("Failed to create scales.csv");
        let task_file    = File::create(task_path).expect("Failed to create task.csv");
        let service_file = File::create(service_path).expect("Failed to create service.csv");

        // Store in entry
        self.entry = Some(Entry {
            scales: scales_file,
            task: task_file,
            service: service_file,
        });
    }

    pub fn log_scales(&mut self, data: &str) {
        if let Some(entry) = &mut self.entry {
            writeln!(entry.scales, "{}", data).expect("Failed to write to scales.csv");
        }
    }

    pub fn log_task(&mut self, data: &str) {
        if let Some(entry) = &mut self.entry {
            writeln!(entry.task, "{}", data).expect("Failed to write to task.csv");
        }
    }

    pub fn log_service(&mut self, data: &str) {
        if let Some(entry) = &mut self.entry {
            writeln!(entry.service, "{}", data).expect("Failed to write to service.csv");
        }
    }
}