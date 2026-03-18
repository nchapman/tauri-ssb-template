use serde::Deserialize;
use tauri::{webview::NewWindowResponse, WebviewUrl, WebviewWindowBuilder};
use url::Url;

#[derive(Deserialize)]
struct TauriConfig {
    plugins: PluginsConfig,
}

#[derive(Deserialize)]
struct PluginsConfig {
    ssb: SsbConfig,
}

#[derive(Deserialize)]
struct SsbConfig {
    url: String,
}

struct SsbState {
    url: Url,
    host: String,
}

fn parse_ssb_config() -> SsbState {
    let config: TauriConfig =
        serde_json::from_str(include_str!("../tauri.conf.json"))
            .expect("Failed to parse plugins.ssb in tauri.conf.json");
    let url = Url::parse(&config.plugins.ssb.url).expect("Invalid plugins.ssb.url");
    let host = url.host_str().expect("plugins.ssb.url has no host").to_string();
    SsbState { url, host }
}

/// Check if the target URL is reachable, then navigate the webview to it.
#[tauri::command]
async fn launch_url(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, SsbState>,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(state.url.as_str())
        .send()
        .await
        .map_err(|e| format!("Cannot reach {}: {e}", state.host))?;

    if !resp.status().is_success() && !resp.status().is_redirection() {
        return Err(format!("Server returned {}", resp.status()));
    }

    window
        .navigate(state.url.clone())
        .map_err(|e| format!("Navigation failed: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ssb = parse_ssb_config();
    let allowed_host = ssb.host.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ssb)
        .invoke_handler(tauri::generate_handler![launch_url])
        .setup(move |app| {
            let config = app.config();

            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title(config.product_name.as_deref().unwrap_or("SSB"))
                .inner_size(1200.0, 800.0)
                .min_inner_size(400.0, 300.0)
                .center()
                .initialization_script(&format!(
                    r#"
                    document.addEventListener('click', function(e) {{
                        const a = e.target.closest('a[href]');
                        if (!a) return;
                        const url = new URL(a.href, window.location.href);
                        if (url.hostname !== '{allowed_host}') {{
                            e.preventDefault();
                            e.stopPropagation();
                            window.open(url.href);
                        }}
                    }}, true);
                    "#
                ))
                .on_new_window(|url, _features| {
                    let _ = tauri_plugin_opener::open_url(url.as_str(), None::<&str>);
                    NewWindowResponse::Deny
                })
                .on_navigation(move |url| {
                    let host = url.host_str().unwrap_or_default();

                    // Allow iframes, blobs, and internal schemes
                    let scheme = url.scheme();
                    if scheme == "about" || scheme == "blob" || scheme == "data" {
                        return true;
                    }

                    // Allow navigation to the SSB's own domain and local dev server
                    if host == allowed_host
                        || host == "localhost"
                        || host == "tauri.localhost"
                    {
                        return true;
                    }

                    // Open everything else in the default browser
                    let _ = tauri_plugin_opener::open_url(url.as_str(), None::<&str>);
                    false
                })
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
