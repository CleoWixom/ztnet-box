const Router = (() => {
  const _routes = [];
  let _current = null;
  let _cleanup = null;

  function match(pattern, path) {
    const pParts = pattern.split('/');
    const rParts = path.split('/');
    if (pParts.length !== rParts.length) return null;
    const params = {};
    for (let i = 0; i < pParts.length; i++) {
      if (pParts[i].startsWith(':')) params[pParts[i].slice(1)] = rParts[i];
      else if (pParts[i] !== rParts[i]) return null;
    }
    return params;
  }

  function resolve() {
    const hash = location.hash.slice(1) || '/dashboard';
    if (hash === _current) return;
    _current = hash;

    // Cleanup previous page
    if (_cleanup) { try { _cleanup(); } catch(e) {} _cleanup = null; }

    // Update active nav item
    document.querySelectorAll('.nav-item').forEach(el => {
      el.classList.toggle('active', el.dataset.route && hash.startsWith(el.dataset.route));
    });

    for (const { pattern, handler } of _routes) {
      const params = match(pattern, hash);
      if (params !== null) {
        const result = handler(params);
        if (result && typeof result.destroy === 'function') _cleanup = result.destroy;
        return;
      }
    }
    // Fallback
    document.getElementById('content').innerHTML =
      '<div class="page"><div class="empty-state"><div class="empty-state-icon">🔍</div><h3>Page not found</h3><p>' + hash + '</p></div></div>';
  }

  return {
    on(pattern, handler) { _routes.push({ pattern, handler }); },
    navigate(path) { location.hash = '#' + path; },
    start() {
      window.addEventListener('hashchange', resolve);
      resolve();
    },
    current() { return _current; },
  };
})();
