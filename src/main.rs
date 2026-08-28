use std::{fs, path::PathBuf};

fn main() {
    // Trying to get the first argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        todo!("Need a default case");
    }
    let to_host = &args[1];

    // Trying to convert it to a path
    let file_path = PathBuf::from(to_host);
    if let Ok(file_contents) = fs::read_to_string(file_path) {
        println!("This is an actual file");
        println!("{}", make_http_message(&file_contents, "text/html"));
    } else {
        eprintln!("File not valid");
    }
}

fn make_http_message(content: &str, content_type: &str) -> String {
    format!(
        "Content-Length: {}\r\n\
        Content-Type: {content_type}\r\n\
        \r\n\
        {content}",
        content.len()
    )
}
