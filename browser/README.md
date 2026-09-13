# Browser Platform (V.04 §18)

Unified Browser Manager for Brave + LibreWolf.

- Profiles: Developer, AI, Digital Twin, Energy, Research.
- Controls: profiles, bookmarks, extensions, certificates, policies,
  privacy config, dashboard URLs, session isolation.
- Internal dashboards may use Tauri/WebView instead of a full browser.

Implementation lives here (future Tauri/Rust manager); `dashboard/` is the
web UI. No privileged browser APIs without authorization.