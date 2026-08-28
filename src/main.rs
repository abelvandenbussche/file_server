use std::{
    error::Error,
    fs, io,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread,
};

fn main() {
    // Trying to get the first argument
    let args: Vec<String> = std::env::args().collect();
    let to_host;
    if args.len() < 2 {
        todo!("Need a default case");
    } else {
        to_host = &args[1];
    }

    // Trying to convert it to a path
    let file_path = PathBuf::from(to_host);
    if let Ok(file_contents) = fs::read_to_string(file_path) {
        let listener = TcpListener::bind("127.0.0.1:5050").expect("Unable to bind tcp listener");
        for stream in listener.incoming() {
            let stream = stream.expect("Failed connection");
            let _ = thread::spawn(move || handle_connection(stream).expect("Thread panicked"));
        }
    } else {
        eprintln!("File not valid");
    }
}

fn make_http_message(status: &str, content: &str, content_type: &str) -> String {
    format!(
        "{status}\r\n\
        Content-Length: {}\r\n\
        Content-Type: {content_type}\r\n\
        \r\n\
        {content}",
        content.len()
    )
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    Ok(())
}
