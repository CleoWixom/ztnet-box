const LogPanel = (() => {
  let _es = null;       // EventSource
  let _streaming = false;
  let _entries = [];
  let _level = 'info';
  let _open = false;
  const MAX_DISPLAY = 300;

  // ── Init ────────────────────────────────────────────────────────────────────

  function init() {
    _renderShell();
    _loadInitial();
  }

  function _renderShell() {
    if (document.getElementById('log-panel')) return;
    const bar = document.createElement('div');
    bar.id = 'log-panel';
    bar.innerHTML = `
      <div id="log-bar" onclick="LogPanel._toggle()">
        <span id="log-bar-label">▲ Logs</span>
        <span id="log-bar-count" class="log-badge">0</span>
        <span id="log-bar-level" class="log-level-badge">info</span>
        <span class="log-bar-spacer"></span>
        <button class="log-btn" onclick="event.stopPropagation();LogPanel._toggleStream()" id="log-stream-btn" title="Start/stop live stream">▶</button>
        <button class="log-btn" onclick="event.stopPropagation();LogPanel._clear()" title="Clear logs">✕</button>
      </div>
      <div id="log-body" style="display:none">
        <div id="log-toolbar">
          <label>Level:
            <select id="log-level-sel" onchange="LogPanel._setLevel(this.value)">
              <option value="error">error</option>
              <option value="warn">warn</option>
              <option value="info" selected>info</option>
              <option value="debug">debug</option>
              <option value="trace">trace</option>
            </select>
          </label>
          <label style="margin-left:1rem">Filter:
            <input id="log-filter" class="log-input" placeholder="substring…" oninput="LogPanel._rerender()">
          </label>
        </div>
        <div id="log-entries"></div>
      </div>`;
    document.body.appendChild(bar);
    _injectStyles();
  }

  // ── Data ────────────────────────────────────────────────────────────────────

  async function _loadInitial() {
    try {
      const data = await api.get('/logs?limit=200');
      _entries = data || [];
      _rerender();
    } catch (e) { /* server may not have logs yet */ }
  }

  function _addEntry(entry) {
    _entries.push(entry);
    if (_entries.length > MAX_DISPLAY) _entries.shift();
    if (_open) _rerender();
    _updateBadge();
  }

  // ── SSE stream ──────────────────────────────────────────────────────────────

  function _startStream() {
    if (_es) return;
    _es = new EventSource('/api/logs/stream');
    _es.addEventListener('log', (e) => {
      try { _addEntry(JSON.parse(e.data)); } catch (_) {}
    });
    _es.onerror = () => { _stopStream(); };
    _streaming = true;
    const btn = document.getElementById('log-stream-btn');
    if (btn) { btn.textContent = '⏹'; btn.title = 'Stop live stream'; }
  }

  function _stopStream() {
    if (_es) { _es.close(); _es = null; }
    _streaming = false;
    const btn = document.getElementById('log-stream-btn');
    if (btn) { btn.textContent = '▶'; btn.title = 'Start live stream'; }
  }

  // ── Render ──────────────────────────────────────────────────────────────────

  const LEVEL_CLASS = {
    error: 'log-error', warn: 'log-warn',
    info: 'log-info', debug: 'log-debug', trace: 'log-trace',
  };

  function _rerender() {
    const container = document.getElementById('log-entries');
    if (!container) return;
    const filter = (document.getElementById('log-filter')?.value || '').toLowerCase();
    const rows = _entries
      .filter(e => !filter || e.message.toLowerCase().includes(filter) || e.target.toLowerCase().includes(filter))
      .slice(-MAX_DISPLAY)
      .map(e => {
        const ts = new Date(e.timestamp).toISOString().replace('T', ' ').slice(0, 23);
        const cls = LEVEL_CLASS[e.level] || '';
        const lvl = (e.level || '').padEnd(5);
        const tgt = _esc(e.target.length > 30 ? '…' + e.target.slice(-28) : e.target);
        const msg = _esc(e.message);
        return `<div class="log-row ${cls}"><span class="log-ts">${ts}</span> <span class="log-lvl">${lvl}</span> <span class="log-tgt">${tgt}</span> <span class="log-msg">${msg}</span></div>`;
      })
      .join('');
    container.innerHTML = rows || '<div class="log-empty">No log entries</div>';
    container.scrollTop = container.scrollHeight;
    _updateBadge();
  }

  function _updateBadge() {
    const el = document.getElementById('log-bar-count');
    if (el) el.textContent = _entries.length;
  }

  function _esc(s) {
    return String(s)
      .replace(/&/g, '&amp;').replace(/</g, '&lt;')
      .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  // ── Actions ─────────────────────────────────────────────────────────────────

  async function _setLevel(level) {
    try {
      await api.put('/logs/level', { level });
      _level = level;
      const badge = document.getElementById('log-bar-level');
      if (badge) badge.textContent = level;
    } catch (e) { Toast.error('Failed to set log level'); }
  }

  async function _clear() {
    try {
      await api.delete('/logs');
      _entries = [];
      _rerender();
    } catch (e) { Toast.error('Failed to clear logs'); }
  }

  // ── CSS ─────────────────────────────────────────────────────────────────────

  function _injectStyles() {
    if (document.getElementById('log-panel-style')) return;
    const s = document.createElement('style');
    s.id = 'log-panel-style';
    s.textContent = `
      #log-panel { position:fixed; bottom:0; left:0; right:0; z-index:900;
        background:var(--bg-secondary,#1a1a2e); border-top:1px solid var(--border,#2a2a4a);
        font-family:var(--font-mono,'Fira Mono',monospace); font-size:12px; }
      #log-bar { display:flex; align-items:center; gap:0.5rem; padding:0.3rem 0.75rem;
        cursor:pointer; user-select:none; color:var(--text-dim,#888);
        background:var(--bg-tertiary,#111122); }
      #log-bar:hover { background:var(--bg-hover,#1e1e3a); }
      .log-badge { background:var(--accent,#6366f1); color:#fff; border-radius:999px;
        padding:0 0.4rem; font-size:10px; min-width:1.4em; text-align:center; }
      .log-level-badge { color:var(--text-dim,#888); font-size:10px;
        border:1px solid var(--border,#2a2a4a); padding:0 0.35rem; border-radius:3px; }
      .log-bar-spacer { flex:1; }
      .log-btn { background:none; border:1px solid var(--border,#2a2a4a); color:var(--text-dim,#888);
        cursor:pointer; padding:0.1rem 0.4rem; border-radius:3px; font-size:11px; }
      .log-btn:hover { background:var(--bg-hover,#1e1e3a); color:var(--text,#eee); }
      #log-body { display:flex; flex-direction:column; height:220px; }
      #log-toolbar { padding:0.35rem 0.75rem; border-bottom:1px solid var(--border,#2a2a4a);
        display:flex; align-items:center; gap:0; color:var(--text-dim,#888); }
      .log-input { background:var(--bg,#0d0d1a); border:1px solid var(--border,#2a2a4a);
        color:var(--text,#eee); padding:0.1rem 0.35rem; border-radius:3px; font-size:11px;
        margin-left:0.3rem; width:160px; }
      #log-level-sel { background:var(--bg,#0d0d1a); border:1px solid var(--border,#2a2a4a);
        color:var(--text,#eee); padding:0.1rem 0.2rem; border-radius:3px; font-size:11px;
        margin-left:0.3rem; }
      #log-entries { flex:1; overflow-y:auto; padding:0.2rem 0; }
      .log-row { padding:0.1rem 0.75rem; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
      .log-row:hover { background:var(--bg-hover,#1e1e3a); }
      .log-ts { color:var(--text-dim,#666); margin-right:0.4rem; }
      .log-lvl { display:inline-block; width:4.5em; font-weight:600; }
      .log-tgt { color:var(--text-dim,#777); margin-right:0.5rem; font-size:11px; }
      .log-msg { color:var(--text,#ddd); }
      .log-error .log-lvl { color:#f87171; }
      .log-warn  .log-lvl { color:#fbbf24; }
      .log-info  .log-lvl { color:#60a5fa; }
      .log-debug .log-lvl { color:#a78bfa; }
      .log-trace .log-lvl { color:#6b7280; }
      .log-empty { color:var(--text-dim,#555); padding:0.5rem 0.75rem; font-style:italic; }
    `;
    document.head.appendChild(s);
  }

  // ── Public ──────────────────────────────────────────────────────────────────

  return {
    init,
    _toggle() {
      _open = !_open;
      const body = document.getElementById('log-body');
      const label = document.getElementById('log-bar-label');
      if (body) body.style.display = _open ? 'flex' : 'none';
      if (label) label.textContent = _open ? '▼ Logs' : '▲ Logs';
      if (_open) _rerender();
    },
    _toggleStream() {
      _streaming ? _stopStream() : _startStream();
    },
    _setLevel,
    _clear,
    _rerender,
  };
})();
