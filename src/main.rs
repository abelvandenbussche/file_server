use notify::{Watcher, recommended_watcher};
use open;
use std::{
    env::args,
    error::Error,
    fs,
    io::{BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

struct Timer {
    start_time: Instant,
    duration: Duration,
}

impl Timer {
    fn start(duration: Duration) -> Self {
        Self {
            start_time: Instant::now(),
            duration,
        }
    }

    fn is_finished(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }

    fn restart(&mut self) {
        self.start_time = Instant::now();
    }
}

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

        if &path == "events" {
            println!("Connection from socket");
            handle_refresh_connection(&mut writer);
            // This is unreachable because of the above function
            break Ok(());
        } else if method == "GET" {
            println!("path: {:?}", path);
            if let Ok(mut contents) = fs::read_to_string(&path) {
                println!("Response: {contents}");
                if path.extension().is_some_and(|ext| ext == "html") {
                    contents = inject_socket_connection(contents);
                }
                send_message(&mut writer, "HTTP/1.1 200 OK", &contents, "text/html")?;
            } else {
                eprintln!("Failed to find file");
                send_message(&mut writer, "HTTP/1.1 404 File Not Found", "", "text/html")?;
            }
        }
    }
}

fn handle_refresh_connection(writer: &mut BufWriter<&TcpStream>) {
    let message = "\
    HTTP/1.1 200 OK\r\n\
    Content-Type: text/event-stream\r\n\
    Cache-Control: no-store\r\n\
    Connection: Keep-alive\r\n\
    \r\n";
    let _ = writer.write_all(message.as_bytes());
    let _ = writer.flush();
    let (tx, rcx) = channel();
    let mut watcher = recommended_watcher(move |res| {
        let _ = tx.send(res);
    })
    .unwrap();

    // We can safely unwrap because the begining of the program already checks that it exists
    let chosen_file = PathBuf::from(args().skip(1).next().unwrap());
    let to_watch_path = if let Some(p) = chosen_file.parent() {
        p.to_owned()
    } else {
        ".".into()
    };

    watcher
        .watch(&to_watch_path, notify::RecursiveMode::Recursive)
        .expect("Failed to watch path");

    let mut timer = Timer::start(Duration::from_millis(100));
    while rcx.recv().is_ok() {
        if timer.is_finished() {
            timer.restart();
            let _ = writer.write_all(b"data: Refresh\r\n\r\n");
            let _ = writer.flush();
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

const TO_INSERT: &str = r#"<script>
    const events = new EventSource("http://localhost:5050/events");
    events.onmessage = (event) => {
        console.log("Server: ", event.data);
        window.location.reload();
    }
</script>"#;

fn inject_socket_connection(html: String) -> String {
    let mut html = html.to_lowercase();
    let insert_index = if let Some(i) = html.find("script") {
        i - 1
    } else if let Some(i) = html.find("head") {
        i + "head".len() + 1
    } else if let Some(i) = html.find("body") {
        i + "body".len() + 1
    } else {
        return html;
    };
    html.insert_str(insert_index, TO_INSERT);
    html
}
