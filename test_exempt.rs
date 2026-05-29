use std::path::Path;

fn is_exempt(line: &str, file_path: &Path) -> bool {
    let line_lower = line.to_lowercase();
    let is_explicit_refusal = line_lower.contains("refusal");
    let is_example_only = line_lower.contains("example-only");
    
    let path_str = file_path.to_string_lossy();
    let is_test_file = path_str.contains("tests") || 
                       path_str.contains("testing") || 
                       path_str.contains("chicago_tdd") || 
                       path_str.contains("mocks") ||
                       path_str.ends_with("mock.rs") ||
                       path_str.contains("cache_trait.rs"); // mock implementations for tests

    is_explicit_refusal || is_example_only || is_test_file
}

fn main() {
    let path = Path::new("/Users/sac/clnrm/crates/clnrm-core/src/chicago_tdd/mod.rs");
    println!("{}", is_exempt("mock", path));
}
