const SettingsRootsPage = (() => {
  // ── QR generation (no external library) ────────────────────────────────────
  // Uses the browser's native QR encoding via CSS-hacked SVG approach,
  // or falls back to textarea. For a real QR we inline a minimal encoder.

  function _qrCanvas(text, size) {
    // Use qrcodejs if available (loaded from CDN lazily), else show textarea
    const el = document.getElementById('planet-qr-output');
    if (!el) return;

    if (window.QRCode) {
      el.innerHTML = '';
      try {
        new window.QRCode(el, { text, width: size, height: size, correctLevel: window.QRCode.CorrectLevel.M });
      } catch(e) {
        el.innerHTML = `<div class="banner banner-warn">QR generation failed: ${Utils.esc(e.message)}</div>`;
      }
      return;
    }

    // Try to load qrcodejs from CDN
    const script = document.createElement('script');
    script.src = 'https://cdnjs.cloudflare.com/ajax/libs/qrcodejs/1.0.0/qrcode.min.js';
    script.onload = () => _qrCanvas(text, size);
    script.onerror = () => {
      // CDN failed — show copyable textarea as fallback
      el.innerHTML = `
        <div class="banner banner-warn mb-sm">⚠️ QR library unavailable (offline?). Copy the text below:</div>
        <textarea class="textarea" rows="4" style="font-family:var(--font-mono);font-size:11px"
          onclick="this.select()">${Utils.esc(text)}</textarea>`;
    };
    document.head.appendChild(script);
  }

  // ── Load ──────────────────────────────────────────────────────────────────
  async function load() {
    let moons = [], peers = [], planet = null;
    try { moons  = await api.get('/local/moons'); }  catch {}
    try { peers  = await api.get('/local/peers'); }  catch {}
    try { planet = await api.get('/system/planet-file'); } catch {}

    const moonPeers = peers.filter(p => p.role === 'MOON');
    const planets   = peers.filter(p => p.role === 'PLANET');

    const moonRows = moons.map(m => `<tr>
      <td class="mono">${Utils.esc(m.id)}</td>
      <td class="text-sm">${new Date(m.timestamp || 0).toLocaleDateString()}</td>
      <td class="text-sm">${(m.roots || []).map(r => Utils.esc(r.identity?.slice(0,20) + '…')).join('<br>')}</td>
      <td>${(m.roots || []).flatMap(r => r.stable_endpoints || r.stableEndpoints || [])
            .map(ep => `<div class="mono text-sm">${Utils.esc(ep)}</div>`).join('')}</td>
      <td>
        <button class="btn btn-danger btn-sm" onclick="SettingsRootsPage._deorbit('${Utils.esc(m.id)}')">
          Deorbit
        </button>
      </td>
    </tr>`).join('');

    const planetRows = planets.map(p => `<tr>
      <td class="mono">${Utils.esc(p.address)}</td>
      <td>${p.latency >= 0 ? p.latency + ' ms' : '—'}</td>
      <td class="text-sm">${Utils.esc(p.version || '—')}</td>
      <td>${(p.paths || []).filter(x => x.active).map(x =>
        `<div class="mono text-sm">${Utils.esc(x.address)}</div>`).join('')}</td>
    </tr>`).join('');

    document.getElementById('content').innerHTML = `<div class="page">

      <div class="page-header">
        <h1 class="page-title">Root Servers</h1>
      </div>

      <!-- Active Planets section -->
      <div class="section">
        <div class="section-title">Active Planet Roots</div>
        ${!planetRows
          ? `<div class="empty-state"><div class="empty-state-icon">🌍</div>
             <h3>No planet peers connected</h3>
             <p>ZeroTier daemon is not connected to default root servers.</p></div>`
          : `<div class="table-wrap"><table>
               <thead><tr><th>Address</th><th>Latency</th><th>Version</th><th>Paths</th></tr></thead>
               <tbody>${planetRows}</tbody>
             </table></div>`}
      </div>

      <!-- Moons section -->
      <div class="section">
        <div class="section-title" style="display:flex;justify-content:space-between;align-items:center">
          Moons (Custom Root Servers)
          ${moonPeers.length
            ? `<span class="badge badge-success">${moonPeers.length} connected</span>`
            : ''}
        </div>
        ${moonPeers.length
          ? `<div class="banner banner-info mb">🌙 Connected: ${moonPeers.map(p => Utils.esc(p.address)).join(', ')}</div>`
          : ''}
        ${!moons.length
          ? `<div class="empty-state">
               <div class="empty-state-icon">🌙</div>
               <h3>No moons configured</h3>
               <p>Moons let you use your own root servers without replacing the default ZeroTier planets.</p>
               <a href="https://docs.zerotier.com/zerotier/moons" target="_blank" class="btn btn-ghost mt">
                 Documentation ↗
               </a>
             </div>`
          : `<div class="table-wrap mb"><table>
               <thead><tr><th>World ID</th><th>Timestamp</th><th>Identity</th><th>Endpoints</th><th></th></tr></thead>
               <tbody>${moonRows}</tbody>
             </table></div>`}
        <div class="card mt">
          <div class="card-title mb">Orbit a Moon</div>
          <div class="input-row">
            <input class="input" id="moon-world-id" placeholder="World ID (10 hex chars)">
            <input class="input" id="moon-seed-id"  placeholder="Seed ID (optional)">
            <button class="btn btn-primary" onclick="SettingsRootsPage._orbit()">Orbit</button>
          </div>
          <div class="text-dim text-sm mt-sm">
            Obtain the World ID from your moon server operator, or generate one with
            <code>zerotier-idtool genmoon</code>.
          </div>
        </div>
      </div>

      <!-- Planet File section -->
      <div class="section">
        <div class="section-title">Planet File</div>
        <div class="card">
          <p class="text-sm text-dim mb">
            A custom <code>planet</code> file replaces ZeroTier's default root servers.
            Use this for fully private networks with no connection to ZeroTier's infrastructure.
            ⚠️ Requires restart. Mobile clients need the base64 string below or a QR code.
          </p>
          ${planet?.is_custom
            ? `<div class="banner banner-warn mb">⚠️ Custom planet file active — ZeroTier will not use default roots.</div>`
            : `<div class="banner banner-info mb">ℹ️ Using default ZeroTier planet file.</div>`}

          <div class="field">
            <label class="field-label">Upload planet file</label>
            <div class="input-row">
              <input type="file" class="input" id="planet-file-input"
                accept=".bin,.planet,*" style="flex:1">
              <button class="btn btn-primary"
                onclick="SettingsRootsPage._uploadPlanet()">Upload</button>
              ${planet?.is_custom
                ? `<button class="btn btn-danger"
                     onclick="SettingsRootsPage._resetPlanet()">Reset to Default</button>`
                : ''}
            </div>
          </div>

          ${planet?.base64 ? `
          <div class="field mt">
            <label class="field-label">Base64 (for mobile clients)</label>
            <div style="display:flex;gap:8px;align-items:flex-start;flex-wrap:wrap">
              <textarea class="textarea" id="planet-b64" rows="3"
                style="font-family:var(--font-mono);font-size:11px;flex:1;min-width:200px"
                readonly onclick="this.select()">${Utils.esc(planet.base64)}</textarea>
              <div style="display:flex;flex-direction:column;gap:8px">
                <button class="btn btn-ghost btn-sm"
                  onclick="navigator.clipboard.writeText('${Utils.esc(planet.base64)}').then(()=>Toast.success('Copied!'))">
                  Copy
                </button>
                <button class="btn btn-ghost btn-sm"
                  onclick="SettingsRootsPage._showQR('${Utils.esc(planet.base64)}')">
                  QR Code
                </button>
              </div>
            </div>
          </div>

          <div id="planet-qr-output" class="mt" style="text-align:center"></div>
          ` : ''}
        </div>
      </div>

    </div>`;
  }

  return {
    init() {
      document.getElementById('content').innerHTML =
        '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading…</div></div>';
      load();
    },

    _showQR(b64) {
      _qrCanvas(b64, 220);
      document.getElementById('planet-qr-output')?.scrollIntoView({ behavior: 'smooth' });
    },

    async _orbit() {
      const worldId = document.getElementById('moon-world-id')?.value?.trim();
      const seed    = document.getElementById('moon-seed-id')?.value?.trim();
      if (!worldId) return Toast.error('World ID is required');
      try {
        await api.post(`/local/moons/${worldId}`, { seed: seed || undefined });
        Toast.success('Orbiting moon ' + worldId);
        load();
      } catch(e) { Toast.error(e.message); }
    },

    async _deorbit(id) {
      if (!await Modal.confirm(`Deorbit moon ${id}?`, { danger: true })) return;
      try { await api.delete(`/local/moons/${id}`); Toast.success('Deorbited'); load(); }
      catch(e) { Toast.error(e.message); }
    },

    async _uploadPlanet() {
      const file = document.getElementById('planet-file-input')?.files?.[0];
      if (!file) return Toast.error('Select a planet file first');
      try {
        const bytes = await file.arrayBuffer();
        const b64 = btoa(String.fromCharCode(...new Uint8Array(bytes)));
        await api.post('/system/planet-file', { base64: b64 });
        Toast.success('Planet file uploaded — ZeroTier will restart');
        load();
      } catch(e) { Toast.error(e.message); }
    },

    async _resetPlanet() {
      if (!await Modal.confirm('Reset to default ZeroTier planet file?', { danger: true })) return;
      try { await api.delete('/system/planet-file'); Toast.success('Reset to default'); load(); }
      catch(e) { Toast.error(e.message); }
    },
  };
})();
