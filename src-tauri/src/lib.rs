use tauri::{webview::NewWindowResponse, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_decorum::WebviewWindowExt;
use url::Url;

fn ssb_host_from_config(config: &tauri::Config) -> String {
    let url_str = config
        .plugins
        .0
        .get("ssb")
        .and_then(|v| v.get("url"))
        .and_then(|v| v.as_str())
        .expect("Missing plugins.ssb.url in tauri.conf.json");
    let url = Url::parse(url_str).expect("plugins.ssb.url must be an absolute URL with a host");
    url.host_str()
        .expect("plugins.ssb.url must be an absolute URL with a host")
        .to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_decorum::init())
        .setup(move |app| {
            let config = app.config();
            let allowed_host = ssb_host_from_config(&config);

            let mut builder = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into()),
            )
            .title(config.product_name.as_deref().unwrap_or("SSB"))
            .inner_size(1200.0, 800.0)
            .min_inner_size(400.0, 300.0);

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
                        style.textContent = `html.tauri-ssb {{ --tauri-ssb-titlebar-height: 32px; }}`;
                        document.documentElement.appendChild(style);

                        // Fixed drag region overlay
                        const tb = document.createElement('div');
                        tb.setAttribute('data-tauri-drag-region', '');
                        tb.style.cssText = 'position:fixed;top:0;left:0;width:100%;height:var(--tauri-ssb-titlebar-height,32px);z-index:2147483647;-webkit-app-region:drag;pointer-events:auto;';
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
                    // Allow internal schemes (tauri:// for local assets, about/blob for iframes)
                    let scheme = url.scheme();
                    if scheme == "tauri" || scheme == "about" || scheme == "blob" {
                        return true;
                    }

                    let host = url.host_str().unwrap_or_default();

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
