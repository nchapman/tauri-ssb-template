# Tauri SSB Template

A Tauri 2 template for wrapping any website as a site-specific browser (SSB) — a lightweight native app that loads a single site with native window chrome, external link handling, and window state persistence.

## Quick Start

1. Clone this repo
2. Edit `src-tauri/tauri.conf.json`:
   - Set `plugins.ssb.url` to your target site
   - Set `productName`, `identifier`, and the remote capability URL under `app.security.capabilities`
   - Replace the icons in `src-tauri/icons/`
3. Install and run:

```sh
pnpm install
pnpm tauri dev
```

## How It Works

The app loads a local loading screen (`index.html`) that immediately redirects to the configured URL via a `<meta http-equiv="refresh">` tag. The URL is baked into the HTML at build time by Vite.

### What's included

- **Navigation guard** — only allows navigation to the configured host. External links open in the default browser.
- **Window state persistence** — remembers size, position, and maximized state across launches.
- **macOS titlebar overlay** — transparent overlay titlebar with traffic light inset. A draggable region is injected into the remote page via initialization script.
- **CSS hook** — the remote page gets a `tauri-ssb` class on `<html>` and a `--ssb-titlebar-height` CSS variable. Use these to make room for the overlay titlebar:

  ```css
  html.tauri-ssb body {
    padding-top: var(--ssb-titlebar-height);
  }
  ```
- **Capabilities** — minimal IPC permissions for the remote site (window controls only), defined inline in `tauri.conf.json`.

## Configuration

All configuration lives in `src-tauri/tauri.conf.json`.

| Field | Purpose |
|---|---|
| `plugins.ssb.url` | The URL to load in the webview |
| `productName` | App name (used as window title) |
| `identifier` | Unique app identifier (reverse domain) |
| `app.security.capabilities[1].remote.urls` | URL pattern for remote IPC permissions (must match your site) |

## Customization

- **Loading screen** — edit `index.html` to change the splash screen shown during redirect.
- **Custom JS on load** — add logic to `src/main.ts` (runs before redirect).
- **Initialization script** — the script injected into the remote page is in `src-tauri/src/lib.rs`. Modify it to add custom CSS, JS, or DOM manipulation on the target site.
- **Camera / Microphone** — both `src-tauri/Info.plist` (privacy descriptions) and `src-tauri/Entitlements.plist` (sandbox entitlements) have commented-out camera and microphone entries. Uncomment both if your site needs them.
- **Window defaults** — size, min size, and titlebar style are configured in `lib.rs`.

## Building

```sh
pnpm tauri build
```

Produces platform-specific installers in `src-tauri/target/release/bundle/`.

### macOS code signing

The template ships with `"signingIdentity": "-"` (ad-hoc signing) so builds can run on other Macs without an Apple Developer certificate. Recipients may need to right-click → Open on first launch to bypass Gatekeeper. Replace with your real signing identity if you have one.
