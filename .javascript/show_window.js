const invoke_loaded = async () => {
    setTimeout(async () => await fetch(await window.__TAURI_INTERNALS__.convertFileSrc("show_window", "actions"), { method: "GET" }), ___CUSTOM___DELAY___);
    window.removeEventListener("DOMContentLoaded", invoke_loaded);
};

window.addEventListener("DOMContentLoaded", invoke_loaded);