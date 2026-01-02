//! Local HTTP callback server for OAuth authentication.
//!
//! Provides a temporary localhost server to receive OAuth callbacks,
//! display a success page to the user, and pass the auth code to the app.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::Duration;
use tracing::{debug, error, info};

/// The port used for the OAuth callback server.
pub const CALLBACK_PORT: u16 = 28491;

/// The full redirect URI for OAuth.
#[allow(dead_code)]
pub fn get_redirect_uri() -> String {
    format!("http://localhost:{}/callback", CALLBACK_PORT)
}

/// Result from the callback server.
pub enum CallbackResult {
    /// Successfully received callback with the full URL.
    Success(String),
    /// Server was cancelled.
    Cancelled,
    /// Error occurred.
    Error(String),
}

/// Start the callback server and wait for a single OAuth callback.
///
/// Returns the full callback URL (including query parameters) when received.
/// The server automatically shuts down after receiving the callback.
pub fn start_callback_server(cancel_rx: mpsc::Receiver<()>) -> CallbackResult {
    let addr = format!("127.0.0.1:{}", CALLBACK_PORT);

    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind callback server to {}: {}", addr, e);
            return CallbackResult::Error(format!("Failed to start server: {}", e));
        }
    };

    // Set non-blocking so we can check for cancellation
    if let Err(e) = listener.set_nonblocking(true) {
        error!("Failed to set non-blocking mode: {}", e);
        return CallbackResult::Error(format!("Server configuration error: {}", e));
    }

    info!("OAuth callback server listening on {}", addr);

    loop {
        // Check for cancellation
        match cancel_rx.try_recv() {
            Ok(()) | Err(mpsc::TryRecvError::Disconnected) => {
                info!("Callback server cancelled");
                return CallbackResult::Cancelled;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }

        // Try to accept a connection
        match listener.accept() {
            Ok((stream, peer_addr)) => {
                debug!("Connection from {}", peer_addr);
                match handle_connection(stream) {
                    Some(url) => {
                        info!("OAuth callback received");
                        return CallbackResult::Success(url);
                    }
                    None => {
                        // Not a valid callback request, continue listening
                        continue;
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No connection yet, sleep briefly and retry
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
                return CallbackResult::Error(format!("Connection error: {}", e));
            }
        }
    }
}

/// Handle an incoming HTTP connection.
///
/// Returns Some(url) if this was a valid OAuth callback, None otherwise.
fn handle_connection(mut stream: TcpStream) -> Option<String> {
    // Set read timeout
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

    let mut buffer = [0; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            debug!("Failed to read request: {}", e);
            return None;
        }
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    debug!("Received request: {}", request.lines().next().unwrap_or(""));

    // Parse the request line to get the path
    let request_line = request.lines().next()?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 {
        send_error_response(&mut stream, 400, "Bad Request");
        return None;
    }

    let method = parts[0];
    let path = parts[1];

    // Only handle GET requests to /callback
    if method != "GET" {
        send_error_response(&mut stream, 405, "Method Not Allowed");
        return None;
    }

    if !path.starts_with("/callback") {
        send_error_response(&mut stream, 404, "Not Found");
        return None;
    }

    // Check if this is an error callback
    if path.contains("error=") {
        send_error_page(&mut stream, path);
        // Still return the URL so the app can handle the error
        return Some(format!("http://localhost:{}{}", CALLBACK_PORT, path));
    }

    // Check if this has the code parameter
    if !path.contains("code=") {
        send_error_response(&mut stream, 400, "Missing authorization code");
        return None;
    }

    // Send success page
    send_success_page(&mut stream);

    // Return the full callback URL
    Some(format!("http://localhost:{}{}", CALLBACK_PORT, path))
}

/// Send a success HTML page.
fn send_success_page(stream: &mut TcpStream) {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Authentication Successful</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            background: white;
            padding: 3rem;
            border-radius: 1rem;
            box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
            text-align: center;
            max-width: 400px;
        }
        .icon {
            width: 80px;
            height: 80px;
            background: #10B981;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 1.5rem;
        }
        .icon svg {
            width: 40px;
            height: 40px;
            stroke: white;
            stroke-width: 3;
            fill: none;
        }
        h1 {
            color: #1F2937;
            font-size: 1.5rem;
            margin-bottom: 0.5rem;
        }
        p {
            color: #6B7280;
            margin-bottom: 1.5rem;
        }
        .hint {
            font-size: 0.875rem;
            color: #9CA3AF;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">
            <svg viewBox="0 0 24 24">
                <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
        </div>
        <h1>Authentication Successful!</h1>
        <p>You have been signed in to Azure PIM.</p>
        <p class="hint">You can close this tab now.</p>
    </div>
</body>
</html>"#;

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

/// Send an error HTML page.
fn send_error_page(stream: &mut TcpStream, path: &str) {
    // Extract error description if present
    let error_desc = if let Some(start) = path.find("error_description=") {
        let start = start + 18;
        let end = path[start..].find('&').map(|i| start + i).unwrap_or(path.len());
        urlencoding::decode(&path[start..end])
            .unwrap_or_else(|_| "Authentication failed".into())
            .to_string()
    } else {
        "Authentication was cancelled or failed.".to_string()
    };

    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Authentication Failed</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .container {{
            background: white;
            padding: 3rem;
            border-radius: 1rem;
            box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
            text-align: center;
            max-width: 400px;
        }}
        .icon {{
            width: 80px;
            height: 80px;
            background: #EF4444;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 1.5rem;
        }}
        .icon svg {{
            width: 40px;
            height: 40px;
            stroke: white;
            stroke-width: 3;
            fill: none;
        }}
        h1 {{
            color: #1F2937;
            font-size: 1.5rem;
            margin-bottom: 0.5rem;
        }}
        p {{
            color: #6B7280;
            margin-bottom: 1.5rem;
        }}
        .hint {{
            font-size: 0.875rem;
            color: #9CA3AF;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">
            <svg viewBox="0 0 24 24">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
        </div>
        <h1>Authentication Failed</h1>
        <p>{}</p>
        <p class="hint">You can close this tab and try again.</p>
    </div>
</body>
</html>"#, error_desc);

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

/// Send an error response.
fn send_error_response(stream: &mut TcpStream, status: u16, message: &str) {
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        message,
        message.len(),
        message
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_redirect_uri() {
        let uri = get_redirect_uri();
        assert_eq!(uri, "http://localhost:28491/callback");
    }
}
