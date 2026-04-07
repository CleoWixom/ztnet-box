const api = (() => {
  const BASE = '/api';

  async function request(method, path, body) {
    const opts = { method, headers: {} };
    if (body !== undefined) {
      opts.headers['Content-Type'] = 'application/json';
      opts.body = JSON.stringify(body);
    }
    const res = await fetch(BASE + path, opts);
    if (res.status === 204) return null;
    const json = await res.json().catch(() => ({ error: res.statusText }));
    if (!res.ok) throw Object.assign(new Error(json.error || 'Request failed'), { code: json.code, status: res.status });
    return json;
  }

  return {
    get:    (path)        => request('GET',    path),
    post:   (path, body)  => request('POST',   path, body),
    put:    (path, body)  => request('PUT',    path, body),
    delete: (path)        => request('DELETE', path),
  };
})();
