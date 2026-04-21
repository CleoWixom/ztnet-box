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

  return { esc, latencyClass };
})();
