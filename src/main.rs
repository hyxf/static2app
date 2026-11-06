use rust_embed::RustEmbed;
use std::env;
use std::net::{TcpListener, UdpSocket};
use std::process::{self, Command};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

const VERSION: &str = match option_env!("STATIC2APP_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

enum CliCommand {
    Run(u16),
    Help,
    Version,
}

fn main() {
    let port = match parse_args(env::args().skip(1)) {
        Ok(CliCommand::Run(port)) => port,
        Ok(CliCommand::Help) => {
            print_help();
            return;
        }
        Ok(CliCommand::Version) => {
            println!("static2app version {VERSION}");
            return;
        }
        Err(message) => {
            eprintln!("Error: {message}\n\nUse --help to see available options.");
            process::exit(2);
        }
    };

    // Bind all interfaces so other devices on the LAN can connect.
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).unwrap_or_else(|err| {
        eprintln!("Failed to bind local server: {err}");
        process::exit(1);
    });
    let address = listener.local_addr().expect("listener address");

    let port = address.port();
    let url = format!("http://127.0.0.1:{port}");
    let server = Server::from_listener(listener, None).unwrap_or_else(|err| {
        eprintln!("Failed to start local server: {err}");
        process::exit(1);
    });

    let browser_url = url.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Err(err) = open_browser(&browser_url) {
            eprintln!("Failed to open browser: {err}");
        }
    });

    eprintln!("Local:   {url}");
    match lan_ip() {
        Some(ip) => eprintln!("Network: http://{ip}:{port}"),
        None => eprintln!("Network: listening on all interfaces at port {port}; find this device's LAN IP to connect"),
    }
    for request in server.incoming_requests() {
        serve(request);
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliCommand, String> {
    let mut port = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            return Ok(CliCommand::Help);
        }
        if arg == "--version" || arg == "-V" {
            return Ok(CliCommand::Version);
        }

        let value = if arg == "--port" {
            args.next()
                .ok_or_else(|| "--port requires a value".to_owned())?
        } else if let Some(value) = arg.strip_prefix("--port=") {
            value.to_owned()
        } else {
            return Err(format!("unknown argument: {arg}"));
        };

        if port.is_some() {
            return Err("--port may only be specified once".to_owned());
        }
        port =
            Some(value.parse::<u16>().map_err(|_| {
                format!("invalid port '{value}'; expected a number from 0 to 65535")
            })?);
    }

    Ok(CliCommand::Run(port.unwrap_or(0)))
}

fn print_help() {
    println!(
        "static2app {VERSION}\n\nUsage: static2app [OPTIONS]\n\nOptions:\n  -h, --help           Print help information\n  -V, --version        Print version information\n      --port <PORT>    Port to listen on (default: random)"
    );
}

// A UDP connect selects the outbound interface without sending a packet. This
// reports the address other devices on the local network can usually reach.
fn lan_ip() -> Option<std::net::IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("192.0.2.1:80").ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_unspecified() && !ip.is_loopback()).then_some(ip)
}

#[cfg(target_os = "macos")]
fn open_browser(url: &str) -> std::io::Result<()> {
    Command::new("open").arg(url).spawn().map(|_| ())
}

#[cfg(target_os = "linux")]
fn open_browser(url: &str) -> std::io::Result<()> {
    Command::new("xdg-open").arg(url).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn open_browser(url: &str) -> std::io::Result<()> {
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map(|_| ())
}

fn serve(request: Request) {
    if request.method() != &Method::Get && request.method() != &Method::Head {
        let _ = request.respond(Response::empty(StatusCode(405)));
        return;
    }

    let method = request.method().clone();
    let raw_path = request.url().split('?').next().unwrap_or("/");
    let Some(decoded_path) = percent_decode_path(raw_path) else {
        let _ = request.respond(Response::empty(StatusCode(400)));
        return;
    };
    let path = decoded_path.trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    let mut asset = Assets::get(path);
    let mut resolved_path = path;
    if asset.is_none() {
        // Missing asset-like paths return 404; extensionless paths are treated as SPA routes.
        if path.rsplit('/').next().unwrap_or(path).contains('.') {
            respond_not_found(request);
            return;
        }
        asset = Assets::get("index.html");
        resolved_path = "index.html";
    }

    let Some(asset) = asset else {
        respond_not_found(request);
        return;
    };

    let mime = mime_type(resolved_path);
    let headers = vec![
        Header::from_bytes("Content-Type", mime).expect("valid content type header"),
        Header::from_bytes("Cache-Control", "no-cache, no-store, must-revalidate").unwrap(),
    ];
    let body = asset.data.into_owned();
    let response = Response::from_data(if method == Method::Head {
        Vec::new()
    } else {
        body
    })
    .with_status_code(200);
    let response = headers
        .into_iter()
        .fold(response, |response, header| response.with_header(header));
    let _ = request.respond(response);
}

fn respond_not_found(request: Request) {
    let _ = request.respond(Response::empty(StatusCode(404)));
}

fn percent_decode_path(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index + 1)?;
            let low = *bytes.get(index + 2)?;
            decoded.push((hex_digit(high)? << 4) | hex_digit(low)?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded).ok()
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn mime_type(path: &str) -> &'static str {
    match path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "webmanifest" => "application/manifest+json",
        "xml" => "application/xml",
        "txt" => "text/plain; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "pdf" => "application/pdf",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "heic" => "image/heic",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "eot" => "application/vnd.ms-fontobject",
        "mp4" | "m4v" => "video/mp4",
        "mpeg" | "mpg" => "video/mpeg",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        "3gp" => "video/3gpp",
        "webm" => "video/webm",
        "ogv" => "video/ogg",
        "mov" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        "wav" => "audio/wav",
        "oga" | "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "opus" => "audio/opus",
        _ => "application/octet-stream",
    }
}
