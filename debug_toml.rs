use std::path::Path;

fn is_toml_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy();
        ext_str == "toml"
    } else if let Some(name) = path.file_name() {
        // Handle .clnrm.toml files and .toml.tera files
        let name_str = name.to_string_lossy();
        name_str.ends_with(".clnrm.toml") || name_str.ends_with(".toml.tera")
    } else {
        false
    }
}

fn main() {
    println!("test.toml: {}", is_toml_file(Path::new("test.toml")));
    println!("test.clnrm.toml: {}", is_toml_file(Path::new("test.clnrm.toml")));
    println!("test.toml.tera: {}", is_toml_file(Path::new("test.toml.tera")));
    println!("test.txt: {}", is_toml_file(Path::new("test.txt")));
    
    // Debug the .toml.tera case
    let path = Path::new("test.toml.tera");
    println!("Path: {:?}", path);
    println!("Extension: {:?}", path.extension());
    println!("File name: {:?}", path.file_name());
    if let Some(name) = path.file_name() {
        let name_str = name.to_string_lossy();
        println!("Name string: {}", name_str);
        println!("Ends with .toml.tera: {}", name_str.ends_with(".toml.tera"));
    }
}
