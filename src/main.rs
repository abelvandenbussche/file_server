use open;
use std::{
    error::Error,
    fs,
    io::{BufRead, BufReader, BufWriter, Write},
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
    let url = format!("http://localhost:5050/{}", file_path.to_string_lossy());
    println!("Succesfully launched server at \x1b[38;2;100;100;200m{url}\x1b[0m",);

    let _ = open::that(url);

    let listener = TcpListener::bind("127.0.0.1:5050").expect("Unable to bind tcp listener");
    for stream in listener.incoming() {
        let stream = stream.expect("Failed connection");
        let _ = thread::spawn(move || handle_connection(stream).expect("Thread panicked"));
    }
}

fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    loop {
        let message = read_message(&mut reader)?;
        let parts: Vec<&str> = message.split_whitespace().collect();

        let method = parts[0];
        let path = PathBuf::from(parts[1]).strip_prefix("/")?.to_owned();
        println!("request: {message}");

        if method == "GET" {
            println!("path: {:?}", path);
            if let Ok(contents) = fs::read_to_string(path) {
                println!("Response: {contents}");
                send_message(&mut writer, "HTTP/1.1 200 OK", &contents, "text/html")?;
            } else {
                eprintln!("Failed to find file");
                send_message(&mut writer, "HTTP/1.1 404 File Not Found", "", "text/html")?;
            }
        }
    }
}

fn read_message(reader: &mut BufReader<&TcpStream>) -> Result<String, Box<dyn Error>> {
    let mut buf = vec![];
    return Ok(loop {
        let mut line = vec![];
        let read = reader.read_until(b'\n', &mut line)?;
        buf.append(&mut line);
        if read == 0 {
            break String::from_utf8_lossy(&buf).to_string();
        }
        if buf.ends_with(b"\r\n\r\n") {
            break String::from_utf8_lossy(&buf).to_string();
        }
    });
}

fn send_message(
    writer: &mut BufWriter<&TcpStream>,
    status_line: &str,
    content: &str,
    content_type: &str,
) -> Result<(), Box<dyn Error>> {
    let message = format!(
        "{status_line}\r\n\
        Content-Length: {}\r\n\
        Content-Type: {}\r\n\
        \r\n\
        {}",
        content.len(),
        content_type,
        content,
    );
    println!("Writing: \'{message}\'");
    writer.write_all(message.as_bytes())?;
    writer.flush()?;
    Ok(())
}
