//! NSApplicationDelegate implementation for handling app lifecycle.

use objc2::mutability::MainThreadOnly;
use objc2::rc::Retained;
use objc2::{declare_class, msg_send_id, ClassType, DeclaredClass};
use objc2_app_kit::NSApplicationDelegate;
use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol};
use tracing::info;

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
    }
);

impl AppDelegate {
    /// Create a new AppDelegate instance.
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}
