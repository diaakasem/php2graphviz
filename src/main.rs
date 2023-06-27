use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    // Get the directory path from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run -- <directory_path>");
        return;
    }

    let directory_path = Path::new(&args[1]);

    // Create the DOT file
    let mut dot_file = fs::File::create("function_calls.dot").expect("Failed to create DOT file");

    // Traverse the PHP project directory
    traverse_directory(directory_path, &mut dot_file);

    let path = Path::new("function_calls.dot");
    remove_duplicate_rules(path);

    println!("DOT file generated successfully.");
}

fn traverse_directory(directory_path: &Path, dot_file: &mut fs::File) {
    // Iterate over the directory entries
    if let Ok(entries) = fs::read_dir(directory_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    // if dir is not "vendor" or "node_modules"
                    if path.file_name().and_then(|name| name.to_str()) != Some("vendor")
                        && path.file_name().and_then(|name| name.to_str()) != Some("node_modules")
                    {
                        // Recursively traverse subdirectories
                        traverse_directory(&path, dot_file);
                    }
                } else if path.extension().and_then(|ext| ext.to_str()) == Some("php") {
                    // Process PHP files
                    process_php_file(&path, dot_file);
                }
            }
        }
    }
}

// TODO: Add support for namespaces
// TODO: Add support for class methods

// it should result in
// digraph functioncalls {
//    // for calls between class methods
//    "class1::function_x" -> "class2::function_y";
//    // for calls from global functions to class methods
//    // if there is a class name, no need for file name
//    "file1::function_z" -> "class1::function_x";
//    // for calls between global functions
//    "file2::function_a" -> "file1::function_z";
// }

fn process_php_file(file_path: &Path, dot_file: &mut fs::File) {
    // Read the contents of the PHP file
    if let Ok(contents) = fs::read_to_string(file_path) {
        // Extract function calls using a simple regex pattern
        let function_calls =
            regex::Regex::new(r#"\b([a-zA-Z_][a-zA-Z0-9_]*::[a-zA-Z_][a-zA-Z0-9_]*)\("#)
                .unwrap()
                .find_iter(&contents)
                .map(|m| m.as_str())
                .collect::<Vec<&str>>();

        // Get the file name without extension
        let file_name = file_path.file_stem().unwrap().to_string_lossy();

        // Write the function calls to the DOT file
        for call in function_calls {
            writeln!(dot_file, r#"    "{}" -> "{}";"#, file_name, call)
                .expect("Failed to write to DOT file");
        }
    } else {
        println!("Failed to read file: {:?}", file_path);
    }
}

fn remove_duplicate_rules(dot_file: &Path) {
    // Read the contents of the DOT file
    println!("Removing duplicate rules : {} ", dot_file.to_str().unwrap());
    if let Ok(contents) = fs::read_to_string(dot_file) {
        println!("contents : {} ", contents);
        // Extract function calls using a simple regex pattern - and deduplicate
        let mut function_calls = regex::Regex::new(
            r#"\s+"([a-zA-Z_][a-zA-Z0-9_:]*)"\s->\s"([a-zA-Z_][a-zA-Z0-9_:]*)\(";"#,
        )
        .unwrap()
        .find_iter(&contents)
        .map(|m| m.as_str().replace("(", ""))
        .collect::<Vec<String>>();
        println!("Removing duplicate rules : {} ", function_calls.len());
        // deduplicate vec<&str>
        function_calls.sort();
        function_calls.dedup();

        //remove file
        fs::remove_file(dot_file).unwrap();

        // Create the DOT file!
        let mut dot_file = fs::File::create(dot_file).expect("Failed to create DOT file");

        // Write the DOT file header
        writeln!(dot_file, "digraph FunctionCalls {{").expect("Failed to write to DOT file");
        // Write the function calls to the DOT file
        for call in function_calls {
            writeln!(dot_file, r#"    {}"#, call).expect("Failed to write to DOT file");
        }
        // Write the DOT file footer
        writeln!(dot_file, "}}").expect("Failed to write to DOT file");
    } else {
        println!("Failed to read file: {:?}", dot_file);
    }
}
