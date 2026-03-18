import { invoke } from "@tauri-apps/api/core";

invoke("launch_url").catch(() => {
  document.querySelector(".spinner")?.classList.add("error");
});
