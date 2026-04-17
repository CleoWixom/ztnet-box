const api = (() => {
  const BASE    = '/api';
  const TIMEOUT = 15000; // 15 s — avoids hanging forever if backend is unresponsive

  async function request(method, path, body) {
    const ctrl  = new AbortController();
    const timer = setTimeout(() => ctrl.abort(), TIMEOUT);
    const opts  = { method, headers: {}, signal: ctrl.signal };
    if (body !== undefined) {
      opts.headers['Content-Type'] = 'application/json';
      opts.body = JSON.stringify(body);
    }
    try {
      const res = await fetch(BASE + path, opts);
      clearTimeout(timer);
      if (res.status === 204) return null;
      const json = await res.json().catch(() => ({ error: res.statusText }));
      if (!res.ok) throw Object.assign(new Error(json.error || 'Request failed'), { code: json.code, status: res.status });
      return json;
    } catch (err) {
      clearTimeout(timer);
      if (err.name === 'AbortError') throw new Error('Request timed out');
      throw err;
    }
  }

  return {
    get:    (path)        => request('GET',    path),
    post:   (path, body)  => request('POST',   path, body),
    put:    (path, body)  => request('PUT',    path, body),
    delete: (path)        => request('DELETE', path),
  };
})();
