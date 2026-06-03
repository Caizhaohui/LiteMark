// LiteMark - Frontend JavaScript
(function() {
  'use strict';

  function featureEnabled(name) {
    return document.body && document.body.dataset[name] !== 'false';
  }

  function scrollSyncEnabled() {
    return featureEnabled('scrollSync');
  }

  // ── WebSocket ──────────────────────────────────────────────────────────
  let ws = null;
  let reconnectTimer = null;
  const RECONNECT_DELAY = 1000;

  function connectWebSocket() {
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${location.host}/ws`);

    ws.onopen = function() {
      console.log('[LiteMark] Connected');
      if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
      }
    };

    ws.onmessage = function(event) {
      try {
        const msg = JSON.parse(event.data);
        handleMessage(msg);
      } catch (e) {
        console.error('[LiteMark] Bad message:', e);
      }
    };

    ws.onclose = function() {
      console.log('[LiteMark] Disconnected, reconnecting...');
      reconnectTimer = setTimeout(connectWebSocket, RECONNECT_DELAY);
    };

    ws.onerror = function(err) {
      console.error('[LiteMark] WebSocket error:', err);
      ws.close();
    };
  }

  function handleMessage(msg) {
    switch (msg.type) {
      case 'init':
      case 'update':
        updatePreview(msg);
        break;
      default:
        console.log('[LiteMark] Unknown message:', msg.type);
    }
  }

  function updatePreview(msg) {
    const container = document.getElementById('preview-content');
    if (container && msg.html) {
      // Remember scroll position
      const scrollRatio = container.scrollHeight > 0
        ? window.scrollY / container.scrollHeight
        : 0;

      container.innerHTML = msg.html;

      // Re-run client-side rendering
      renderMath();
      renderMermaid();
      renderHighlight();
      makeTaskCheckboxesInteractive();

      // Restore scroll position approximately
      if (msg.type === 'update' && scrollSyncEnabled()) {
        requestAnimationFrame(function() {
          window.scrollTo(0, scrollRatio * container.scrollHeight);
        });
      }
    }

    if (msg.title) {
      document.title = msg.title + ' - LiteMark';
    }
  }

  // ── KaTeX Math Rendering ───────────────────────────────────────────────
  function renderMath() {
    if (!featureEnabled('renderMath')) return;
    if (typeof renderMathInElement !== 'undefined') {
      renderMathInElement(document.getElementById('preview-content') || document.body, {
        delimiters: [
          { left: '$$', right: '$$', display: true },
          { left: '$', right: '$', display: false },
          { left: '\\(', right: '\\)', display: false },
          { left: '\\[', right: '\\]', display: true }
        ],
        throwOnError: false
      });
    }
  }

  // ── Mermaid Diagram Rendering ──────────────────────────────────────────
  let mermaidInitialized = false;

  function renderMermaid() {
    if (!featureEnabled('renderMermaid')) return;
    if (typeof mermaid === 'undefined') return;

    if (!mermaidInitialized) {
      mermaid.initialize({ startOnLoad: false, theme: 'default' });
      mermaidInitialized = true;
    }

    const diagrams = document.querySelectorAll('pre.mermaid:not([data-processed])');
    diagrams.forEach(function(el, i) {
      const id = 'mermaid-' + Date.now() + '-' + i;
      const code = el.textContent;
      try {
        mermaid.render(id, code).then(function(result) {
          el.innerHTML = result.svg;
          el.setAttribute('data-processed', 'true');
        }).catch(function(err) {
          el.innerHTML = '<pre style="color:red;">Mermaid error: ' + err.message + '</pre>';
          el.setAttribute('data-processed', 'true');
        });
      } catch (e) {
        // mermaid v10+ uses async render
      }
    });
  }

  // ── Syntax Highlighting ────────────────────────────────────────────────
  function renderHighlight() {
    if (!featureEnabled('renderHighlight')) return;
    if (typeof hljs !== 'undefined') {
      document.querySelectorAll('pre code[class*="language-"]').forEach(function(el) {
        hljs.highlightElement(el);
      });
    }
  }

  // ── Task List Checkbox Interaction ──────────────────────────────────────
  function makeTaskCheckboxesInteractive() {
    document.querySelectorAll('input[type="checkbox"][data-task-line]').forEach(function(cb) {
      cb.disabled = false;
      cb.addEventListener('change', function() {
        const line = parseInt(cb.getAttribute('data-task-line'));
        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({ type: 'taskToggle', line: line }));
        }
      });
    });
  }

  // ── Scroll Sync ────────────────────────────────────────────────────────
  function scrollToLine(line) {
    const el = document.querySelector('[data-source-line="' + line + '"]');
    if (el) {
      const rect = el.getBoundingClientRect();
      const offset = window.scrollY + rect.top - 80;
      window.scrollTo({ top: offset, behavior: 'smooth' });
    } else {
      // Try finding nearest element with data-source-line <= line
      let best = null;
      let bestLine = -1;
      document.querySelectorAll('[data-source-line]').forEach(function(el) {
        const elLine = parseInt(el.getAttribute('data-source-line'));
        if (elLine <= line && elLine > bestLine) {
          bestLine = elLine;
          best = el;
        }
      });
      if (best) {
        const rect = best.getBoundingClientRect();
        const offset = window.scrollY + rect.top - 80;
        window.scrollTo({ top: offset, behavior: 'smooth' });
      }
    }
  }

  // ── Theme Toggle ───────────────────────────────────────────────────────
  function toggleTheme() {
    const html = document.documentElement;
    const current = html.getAttribute('data-theme');
    const next = current === 'github-dark' ? 'github-light' : 'github-dark';
    html.setAttribute('data-theme', next);

    // Update theme CSS link
    const link = document.querySelector('link[href*="themes/"]');
    if (link) {
      link.href = '/assets/themes/' + next + '.css';
    }
  }

  // ── Image Lightbox ─────────────────────────────────────────────────────
  function initLightbox() {
    if (!featureEnabled('renderLightbox')) return;
    document.addEventListener('click', function(e) {
      if (e.target.tagName === 'IMG' && e.target.closest('.litemark-content')) {
        e.preventDefault();
        const overlay = document.createElement('div');
        overlay.className = 'lightbox-overlay';
        overlay.innerHTML = '<img src="' + e.target.src + '" alt="' + (e.target.alt || '') + '">';
        overlay.addEventListener('click', function() {
          document.body.removeChild(overlay);
        });
        document.addEventListener('keydown', function handler(ev) {
          if (ev.key === 'Escape') {
            if (overlay.parentNode) document.body.removeChild(overlay);
            document.removeEventListener('keydown', handler);
          }
        });
        document.body.appendChild(overlay);
      }
    });
  }

  // ── Initialize ─────────────────────────────────────────────────────────
  document.addEventListener('DOMContentLoaded', function() {
    connectWebSocket();
    initLightbox();
    makeTaskCheckboxesInteractive();

    // Initial render for static exports
    renderMath();
    renderMermaid();
    renderHighlight();

    // Keyboard shortcuts
    document.addEventListener('keydown', function(e) {
      if (e.ctrlKey && e.key === 'd') {
        e.preventDefault();
        toggleTheme();
      }
    });
  });
})();
