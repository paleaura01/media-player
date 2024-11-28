// renderer/renderer.js

import { loadPlaylists } from "./playlists.js";
import { setupUIListeners, renderPlaylists } from "./ui.js";

window.addEventListener("DOMContentLoaded", () => {
  loadPlaylists();
  renderPlaylists();
  setupUIListeners();
});
