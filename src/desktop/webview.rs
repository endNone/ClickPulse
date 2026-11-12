//! Keep every script entry point behind the same native visibility gate.
use std::cell::Cell;

pub(super) struct VisibleWebView {
    pub inner: wry::WebView,
    active: Cell<bool>,
}
impl VisibleWebView {
    pub fn new(inner: wry::WebView, active: bool) -> Self {
        Self {
            inner,
            active: Cell::new(active),
        }
    }
    pub fn is_active(&self) -> bool {
        self.active.get()
    }
    pub fn set_active(&self, active: bool) {
        self.active.set(active);
    }
    pub fn evaluate_script(&self, script: &str) -> wry::Result<()> {
        if self.is_active() {
            self.inner.evaluate_script(script)
        } else {
            Ok(())
        }
    }
    /// Stop animation/polling while still visible, before the native hide.
    pub fn suspend(&self) {
        let _ = self.evaluate_script("window.receiveView?.({visible:false});");
        self.set_active(false);
    }
}
