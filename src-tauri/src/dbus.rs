use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter, Manager};
use zbus::interface;

pub struct FotosService {
    app: AppHandle,
    is_capturing: Arc<AtomicBool>,
}

#[interface(name = "io.github.charly.Fotos")]
impl FotosService {
    async fn activate(&self) -> zbus::fdo::Result<()> {
        if let Some(window) = self.app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
        Ok(())
    }

    async fn take_screenshot(&self, mode: String) -> zbus::fdo::Result<String> {
        if self.is_capturing.load(Ordering::SeqCst) {
            return Ok("Failed".to_string());
        }
        let event = match mode.as_str() {
            "region" => "global-capture-region",
            "fullscreen" => "global-capture-fullscreen",
            _ => return Ok("InvalidArgs".to_string()),
        };
        let _ = self.app.emit(event, ());
        Ok("Ok".to_string())
    }

    #[zbus(property)]
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
}

pub async fn start_service(app: AppHandle, is_capturing: Arc<AtomicBool>) -> anyhow::Result<()> {
    let service = FotosService { app, is_capturing };
    let _conn = zbus::connection::Builder::session()?
        .name("io.github.charly.Fotos")?
        .serve_at("/io/github/charly/Fotos", service)?
        .build()
        .await?;
    // Keep the connection alive for the lifetime of the app.
    std::future::pending::<()>().await;
    unreachable!()
}
