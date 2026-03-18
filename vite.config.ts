import { defineConfig } from "vite";
import { readFileSync } from "fs";
import { dirname, resolve } from "path";
import { fileURLToPath } from "url";
import { viteSingleFile } from "vite-plugin-singlefile";

const __dirname = dirname(fileURLToPath(import.meta.url));
const host = process.env.TAURI_DEV_HOST;
const tauriConf = JSON.parse(readFileSync(resolve(__dirname, "src-tauri/tauri.conf.json"), "utf-8"));
const ssbUrl = tauriConf?.plugins?.ssb?.url;
if (!ssbUrl) {
  throw new Error(
    'Missing plugins.ssb.url in src-tauri/tauri.conf.json. ' +
    'Add: { "plugins": { "ssb": { "url": "https://your-site.com/" } } }'
  );
}

process.env.VITE_SSB_URL = ssbUrl;

export default defineConfig({
  plugins: [viteSingleFile()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
