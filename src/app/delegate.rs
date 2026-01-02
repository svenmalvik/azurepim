//! NSApplicationDelegate implementation for handling app lifecycle and URL callbacks.

use objc2::mutability::MainThreadOnly;
use objc2::rc::Retained;
use objc2::{declare_class, msg_send_id, ClassType, DeclaredClass};
use objc2_app_kit::{NSApplication, NSApplicationDelegate};
use objc2_foundation::{MainThreadMarker, NSArray, NSNotification, NSObject, NSObjectProtocol, NSURL};
use tokio::sync::mpsc;
use tracing::{error, info};

/// Channel sender for OAuth callback URLs.
/// Set by main.rs during initialization.
static CALLBACK_SENDER: std::sync::OnceLock<mpsc::Sender<String>> = std::sync::OnceLock::new();

/// Initialize the callback channel.
pub fn init_callback_channel() -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel(1);
    CALLBACK_SENDER
        .set(tx)
        .expect("Callback channel already initialized");
    rx
}

/// Get the callback sender (for testing).
#[allow(dead_code)]
pub fn get_callback_sender() -> Option<&'static mpsc::Sender<String>> {
    CALLBACK_SENDER.get()
}

// Define the AppDelegate class
declare_class!(
    pub struct AppDelegate;

    unsafe impl ClassType for AppDelegate {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "AzurePimAppDelegate";
    }

    impl DeclaredClass for AppDelegate {}

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[method(applicationDidFinishLaunching:)]
        fn application_did_finish_launching(&self, _notification: &NSNotification) {
            info!("Application did finish launching");
        }

        #[method(applicationWillTerminate:)]
        fn application_will_terminate(&self, _notification: &NSNotification) {
            info!("Application will terminate");
        }

        /// Handle URLs opened with this application (OAuth callbacks).
        ///
        /// This is called when macOS opens a URL like `azurepim://callback?code=xxx`.
        #[method(application:openURLs:)]
        fn application_open_urls(&self, _application: &NSApplication, urls: &NSArray<NSURL>) {
            let count = urls.len();
            info!("Received {} URL(s) from system", count);
            for i in 0..count {
                // SAFETY: Index is within bounds (0..count), and NSURL is a valid object type
                unsafe {
                    let url = urls.objectAtIndex(i);
                    if let Some(url_string) = url.absoluteString() {
                        handle_url(&url_string.to_string());
                    }
                }
            }
        }
    }
);

impl AppDelegate {
    /// Create a new AppDelegate instance.
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}

/// Handle an incoming URL from the URL scheme handler.
///
/// Called by `application:openURLs:` when macOS opens a URL like `azurepim://callback?code=xxx`.
pub fn handle_url(url_string: &str) {
    // Don't log the full URL - it contains the authorization code
    info!("Received OAuth callback URL");

    if !url_string.starts_with("azurepim://callback") {
        info!("Ignoring non-callback URL");
        return;
    }

    // Send the URL to the waiting auth flow
    if let Some(sender) = CALLBACK_SENDER.get() {
        // Use try_send since we might not be in an async context
        match sender.try_send(url_string.to_string()) {
            Ok(_) => info!("OAuth callback dispatched to handler"),
            Err(_) => error!("Failed to send OAuth callback: channel error"),
        }
    } else {
        error!("No callback handler registered for OAuth URL");
    }
}

/// Register the URL scheme handler.
///
/// Note: URL scheme handling is done via the Info.plist CFBundleURLTypes configuration.
/// When macOS opens a `azurepim://` URL, it will launch this app.
/// The URL is then received via NSApplicationDelegate's `application:openURLs:` method
/// or via Apple Events.
///
/// For this implementation, URLs are delivered by the system to our app delegate.
#[allow(dead_code)]
pub fn register_url_handler(_mtm: MainThreadMarker) {
    info!("URL scheme handler registered via Info.plist");
    // URL handling is configured in Info.plist with CFBundleURLTypes
    // The URL will be delivered via application:openURLs: delegate method
    // or we can handle it via command-line arguments when the app is launched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_url_non_callback() {
        // This should not panic, just log and return
        handle_url("https://example.com");
    }
}
