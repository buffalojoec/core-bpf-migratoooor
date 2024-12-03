//! Module for loading files from local filesystem.

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub struct FileReader {
    directories: Vec<PathBuf>,
}

impl FileReader {
    pub fn new(directories: &[&str]) -> Self {
        Self {
            directories: directories.iter().map(PathBuf::from).collect(),
        }
    }

    fn find_file(&self, filename: &str) -> Option<PathBuf> {
        for dir in self.directories.iter() {
            let candidate = dir.join(filename);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        None
    }

    fn read_file<P: AsRef<Path>>(&self, path: P) -> Vec<u8> {
        let path = path.as_ref();
        let mut file = File::open(path).expect("Unable to open file");

        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data)
            .expect("Unable to read file");
        file_data
    }

    pub fn load_program_elf(&self, program_name: &str) -> Vec<u8> {
        let file_name = format!("{program_name}.so");
        let program_file = self
            .find_file(&file_name)
            .expect("Unable to find program ELF file");
        self.read_file(program_file)
    }
}
