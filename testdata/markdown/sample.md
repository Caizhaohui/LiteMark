---
title: LiteMark Render Spike
---

# LiteMark Render Spike

This document exercises **bold**, *italic*, `inline code`, and a [link](https://example.com).

## Mermaid Diagram

```mermaid
graph LR
    A[Markdown] --> B(crossnote)
    B --> C{HTML}
    C -->|safe| D[Webview]
```

## KaTeX Math

Inline equation: $E = mc^2$

Block equation:

$$
\int_a^b f(x)\,dx = F(b) - F(a)
$$

## Code Block

```ts
function greet(name: string): string {
  return `Hello, ${name}!`;
}
```

## Task List

- [x] Repository scaffold
- [x] Sidecar ping
- [ ] Real preview UI (M2)

> Untrusted content is sanitized; scripts stay off.
