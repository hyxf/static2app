use rust_embed::RustEmbed;
use std::env;
use std::net::TcpListener;
use std::process::{self, Command};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

const VERSION: &str = match option_env!("STATIC2APP_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

fn main() {
    if env::args().skip(1).any(|arg| arg == "--version") {
        println!("static2app version {VERSION}");
        return;
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|err| {
        eprintln!("Failed to bind local server: {err}");
        process::exit(1);
    });
    let address = listener.local_addr().expect("listener address");

    let url = format!("http://{address}");
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

    eprintln!("Server running at {url} (version: {VERSION})");
    for request in server.incoming_requests() {
        serve(request);
    }
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
    Command::new("cmd").args(["/C", "start", "", url]).spawn().map(|_| ())
}

fn serve(request: Request) {
    if request.method() != &Method::Get && request.method() != &Method::Head {
        let _ = request.respond(Response::empty(StatusCode(405)));
        return;
    }

    let method = request.method().clone();
    let path = request.url().split('?').next().unwrap_or("/").trim_start_matches('/');
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
        .with_status_code(200)
        .with_headers(headers);
    let _ = request.respond(response);
}

fn respond_not_found(request: Request) {
    let _ = request.respond(Response::empty(StatusCode(404)));
}

fn mime_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => "application/octet-stream",
    }
}
