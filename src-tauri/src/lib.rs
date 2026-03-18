use serde::Deserialize;
use tauri::{webview::NewWindowResponse, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_decorum::WebviewWindowExt;
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
        .plugin(tauri_plugin_decorum::init())
        .manage(ssb)
        .invoke_handler(tauri::generate_handler![launch_url])
        .setup(move |app| {
            let config = app.config();

            let mut builder = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into()),
            )
            .title(config.product_name.as_deref().unwrap_or("SSB"))
            .inner_size(1200.0, 800.0)
            .min_inner_size(400.0, 300.0)
            .center();

            #[cfg(target_os = "macos")]
            {
                builder = builder
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .hidden_title(true);
            }

            let window = builder
                .initialization_script(&format!(
                    r#"
                    // Mark the page as running inside the SSB
                    document.documentElement.classList.add('tauri-ssb');

                    // Default SSB styles
                    (function() {{
                        // Expose titlebar height as a CSS variable for sites to use
                        const style = document.createElement('style');
                        style.textContent = `html.tauri-ssb {{ --ssb-titlebar-height: 32px; }}`;
                        document.documentElement.appendChild(style);

                        // Fixed drag region overlay
                        const tb = document.createElement('div');
                        tb.setAttribute('data-tauri-drag-region', '');
                        tb.style.cssText = 'position:fixed;top:0;left:0;width:100%;height:var(--ssb-titlebar-height,32px);z-index:2147483647;-webkit-app-region:drag;pointer-events:auto;';
                        document.documentElement.appendChild(tb);
                    }})();

                    // Open external links in the default browser
                    document.addEventListener('click', function(e) {{
                        const a = e.target.closest('a[href]');
                        if (!a) return;
                        try {{
                            const url = new URL(a.href, window.location.href);
                            if (!['http:', 'https:', 'mailto:', 'tel:'].includes(url.protocol)) return;
                            if (url.hostname !== '{allowed_host}') {{
                                e.preventDefault();
                                e.stopPropagation();
                                window.open(url.href);
                            }}
                        }} catch (_) {{}}
                    }}, true);
                    "#
                ))
                .on_new_window(|url, _features| {
                    let scheme = url.scheme();
                    if matches!(scheme, "http" | "https" | "mailto" | "tel") {
                        let _ = tauri_plugin_opener::open_url(url.as_str(), None::<&str>);
                    }
                    NewWindowResponse::Deny
                })
                .on_navigation(move |url| {
                    let host = url.host_str().unwrap_or_default();

                    // Allow internal schemes (tauri:// for local assets, about/blob for iframes)
                    let scheme = url.scheme();
                    if scheme == "tauri" || scheme == "about" || scheme == "blob" {
                        return true;
                    }

                    // Allow navigation to the SSB's own domain and local dev server
                    if host == allowed_host || host == "tauri.localhost" {
                        return true;
                    }

                    #[cfg(debug_assertions)]
                    if host == "localhost" {
                        return true;
                    }

                    // Open everything else in the default browser
                    if matches!(scheme, "http" | "https" | "mailto" | "tel") {
                        let _ = tauri_plugin_opener::open_url(url.as_str(), None::<&str>);
                    }
                    false
                })
                .build()?;

            window
                .create_overlay_titlebar()
                .map_err(|e| e.to_string())?;

            #[cfg(target_os = "macos")]
            window
                .set_traffic_lights_inset(12.0, 16.0)
                .map_err(|e| e.to_string())?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
