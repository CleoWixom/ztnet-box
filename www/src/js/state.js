const State = (() => {
  const _store = {
    nodeStatus:      null,
    networks:        [],
    peers:           [],
    metrics:         null,
    metricsStatus:   null,
    activeNetworkId: null,
    tokens:          [],
    moons:           [],
    controllerNets:  [],
    exitnodeStatus:  null,
    config:          null,
  };
  const _listeners = {};

  return {
    get(key)         { return _store[key]; },
    set(key, value)  {
      _store[key] = value;
      (_listeners[key] || []).forEach(fn => fn(value));
    },
    on(key, fn)      {
      if (!_listeners[key]) _listeners[key] = [];
      _listeners[key].push(fn);
      return () => { _listeners[key] = _listeners[key].filter(f => f !== fn); };
    },
  };
})();

// ── Utils ──────────────────────────────────────────────────────────────────────
// Shared utilities available to all components and pages.

const Utils = (() => {
  /** Escape a value for safe HTML insertion (escapes &, <, >, "). */
  function esc(s) {
    return String(s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  /** CSS class for peer latency colouring. */
  function latencyClass(ms) {
    if (ms < 50)  return 'latency-good';
    if (ms < 150) return 'latency-medium';
    return 'latency-bad';
  }

  /**
   * Collapsible help section (RD2-2/3/4).
   */
  function helpSection(id, title, content) {
    return `
      <details class="help-section" id="${id}">
        <summary class="help-summary">
          <span class="help-icon">?</span> ${Utils.esc(title)}
        </summary>
        <div class="help-body">${content}</div>
      </details>`;
  }

  /**
   * Standard loading placeholder for page init().
   * @param {string=} title - optional page title shown while loading
   */
  function pageLoading(title) {
    return `<div class="page">
      ${title ? `<div class="page-header"><h1 class="page-title">${Utils.esc(title)}</h1></div>` : ''}
      <div class="loading-row"><div class="spinner"></div> Loading…</div>
    </div>`;
  }

  /**
   * Standard error state for page-level failures.
   */
  function pageError(message) {
    return `<div class="page"><div class="banner banner-danger">❌ ${Utils.esc(message)}</div></div>`;
  }

  return { esc, latencyClass, helpSection, pageLoading, pageError };
})();
