use std::path::Path;

pub trait RunJavaClassVerifier {
    type Error;
    fn run(&self, entrance_class: &str, class_dir: &Path) -> Result<Vec<u8>, Self::Error>;
    fn get_current_unwind(&self) -> Option<usize>;
    fn grow_unwind(&mut self, err_loops: &Vec<String>) -> Result<(), Self::Error>;
}