import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import tauriConf from "../src-tauri/tauri.conf.json";
import logo from "./assets/logo.svg";
import "./App.css";

type Status = "loading" | "offline";

function hostname(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return url;
  }
}

function App() {
  const [status, setStatus] = useState<Status>("loading");

  const attemptLoad = () => {
    setStatus("loading");
    invoke("launch_url")
      .catch(() => setStatus("offline"));
  };

  useEffect(() => {
    attemptLoad();
  }, []);

  return (
    <div className="splash">
      <img
        src={logo}
        alt={`${tauriConf.productName} logo`}
        className={`splash-logo ${status === "loading" ? "pulse" : ""}`}
      />
      <h1 className="splash-name">{tauriConf.productName}</h1>

      {status === "loading" && (
        <div className="splash-status">
          <div className="spinner" />
          <span>Loading&hellip;</span>
        </div>
      )}

      {status === "offline" && (
        <div className="splash-status">
          <p className="offline-message">
            Unable to reach {hostname(tauriConf.plugins.ssb.url)}
          </p>
          <button className="retry-button" onClick={attemptLoad}>
            Retry
          </button>
        </div>
      )}
    </div>
  );
}

export default App;
