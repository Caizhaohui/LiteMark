# LiteMark Privacy

LiteMark is a **local-first** desktop application.

## What we collect

**Nothing.** LiteMark does not include telemetry, analytics, crash-upload
services, accounts, or cloud sync.

## What stays on your machine

- Document content you open or edit
- Recovery snapshots under `%LOCALAPPDATA%\LiteMark\LiteMark\recovery\`
- Recent files list and settings under the same app-data root
- Optional crash diagnostic text files you export manually (Settings → Export crash report)

## Network

By default LiteMark does not open outbound network connections for core editing.

Optional features that may use the network only when you initiate them:

- Opening `http://` / `https://` links (via the system browser)
- Automatic updates **only if** you configure an update endpoint (disabled by default)

Remote diagram services (PlantUML server, Kroki, etc.) are **off** by default.

## Third parties

Rendering uses local libraries (crossnote, KaTeX, Mermaid pipeline). PDF export
uses your installed Edge/Chrome. Pandoc export uses a local Pandoc install if
present. No document content is uploaded to LiteMark servers — there are none.
