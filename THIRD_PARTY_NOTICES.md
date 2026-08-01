# Third-Party Notices

LiteMark includes or depends on third-party software. This file lists the major
components and their licenses. Full license texts for crossnote / Markdown
Preview Enhanced lineage are also under `licenses/`.

LiteMark is **not** affiliated with, endorsed by, or sponsored by the authors of
Markdown Preview Enhanced, Typora, or crossnote.

---

## crossnote

- Project: https://github.com/shd101wyy/crossnote
- License: University of Illinois/NCSA Open Source License
- Used for: Markdown enhanced rendering, HTML/PDF export pipeline
- See: `licenses/crossnote-LICENSE.md`

## Tauri

- Project: https://github.com/tauri-apps/tauri
- License: Apache-2.0 / MIT
- Used for: Desktop application shell (Windows WebView2)

## React

- Project: https://github.com/facebook/react
- License: MIT
- Used for: Application UI

## Monaco Editor

- Project: https://github.com/microsoft/monaco-editor
- License: MIT
- Used for: Source-mode Markdown editing

## DOMPurify

- Project: https://github.com/cure53/DOMPurify
- License: Apache-2.0 / MPL-2.0
- Used for: Defense-in-depth HTML sanitization of preview output

## KaTeX

- Project: https://github.com/KaTeX/KaTeX
- License: MIT
- Used for: Math formula styles in the preview pane

## puppeteer-core

- Project: https://github.com/puppeteer/puppeteer
- License: Apache-2.0
- Used for: PDF printing via system Edge/Chrome (no bundled Chromium)

## Other Rust / npm dependencies

Additional transitive dependencies are declared in `src-tauri/Cargo.lock` and
`pnpm-lock.yaml` with their respective open-source licenses.
