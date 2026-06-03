# LiteMark - Fetch vendor assets for offline support
# Run this after clone to populate assets/vendor/ with KaTeX, Mermaid, highlight.js
# Usage: pwsh -File scripts/fetch-vendor-assets.ps1

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$vendorDir = Join-Path $scriptDir "..\assets\vendor"
New-Item -ItemType Directory -Force -Path $vendorDir | Out-Null

Write-Host "Fetching LiteMark vendor assets into $vendorDir ..." -ForegroundColor Cyan

# KaTeX (math) - v0.16.9 stable
Write-Host "  - KaTeX JS, CSS, auto-render..."
Invoke-WebRequest -Uri "https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js" -OutFile (Join-Path $vendorDir "katex.min.js") -UseBasicParsing
Invoke-WebRequest -Uri "https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css" -OutFile (Join-Path $vendorDir "katex.min.css") -UseBasicParsing
Invoke-WebRequest -Uri "https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js" -OutFile (Join-Path $vendorDir "auto-render.min.js") -UseBasicParsing

# Mermaid (diagrams) - v10 (compatible with app.js v10+ async render)
Write-Host "  - Mermaid..."
Invoke-WebRequest -Uri "https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js" -OutFile (Join-Path $vendorDir "mermaid.min.js") -UseBasicParsing

# highlight.js (syntax) - v11, common bundle + github style (works well with our themes)
Write-Host "  - highlight.js + CSS..."
Invoke-WebRequest -Uri "https://cdn.jsdelivr.net/npm/highlight.js@11/lib/common.min.js" -OutFile (Join-Path $vendorDir "highlight.min.js") -UseBasicParsing
Invoke-WebRequest -Uri "https://cdn.jsdelivr.net/npm/highlight.js@11/styles/github.min.css" -OutFile (Join-Path $vendorDir "highlight.min.css") -UseBasicParsing

Write-Host "Done! Assets ready for full offline preview/export (math, diagrams, highlighting)." -ForegroundColor Green
Write-Host "You can re-run this script to update versions." -ForegroundColor Yellow
