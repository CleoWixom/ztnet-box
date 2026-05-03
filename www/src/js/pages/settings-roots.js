const SettingsRootsPage = (() => {
  async function load() {
    let moons = [], peers = [];
    try { moons = await api.get('/local/moons'); } catch(e){}
    try { peers = await api.get('/local/peers'); } catch(e){}
    const moonPeers = peers.filter(p => p.role === 'MOON');

    const rows = moons.map(m => `<tr>
        <td class="mono">${Utils.esc(m.id)}</td>
        <td class="text-sm">${new Date(m.timestamp).toLocaleDateString()}</td>
        <td class="text-sm">${(m.roots||[]).map(r=>Utils.esc(r.identity?.slice(0,20)+'…')).join('<br>')}</td>
        <td>
          <button class="btn btn-ghost btn-sm" onclick="SettingsRootsPage._showQr('${Utils.esc(m.id)}')" title="Show QR code">QR</button>
          <button class="btn btn-danger btn-sm" onclick="SettingsRootsPage._remove('${Utils.esc(m.id)}')">Remove</button>
        </td>
      </tr>`).join('');

    document.getElementById('content').innerHTML = `<div class="page">
        <div class="page-header">
          <h1 class="page-title">Root Servers</h1>
        </div>

        ${moonPeers.length ? `<div class="banner banner-info">🌙 ${moonPeers.length} moon peer${moonPeers.length!==1?'s':''} connected: ${moonPeers.map(p=>Utils.esc(p.address)).join(', ')}</div>` : ''}

        <div class="section">
          <div class="section-title">Moons</div>
          ${!moons.length
            ? `<div class="empty-state">
                 <div class="empty-state-icon">🌙</div>
                 <h3>No moons configured</h3>
                 <p>Moons let you use your own root servers instead of the default ZeroTier planets.</p>
                 <a href="https://docs.zerotier.com/zerotier/moons" target="_blank" class="btn btn-ghost mt">Documentation ↗</a>
               </div>`
            : `<div class="table-wrap">
                 <table>
                   <thead><tr><th>World ID</th><th>Timestamp</th><th>Roots</th><th></th></tr></thead>
                   <tbody>${rows}</tbody>
                 </table>
               </div>`}
        </div>

        <div class="section">
          <div class="section-title">Orbit a Moon</div>
          <div class="card">
            <div class="field-row">
              <div class="field" style="flex:1">
                <label class="field-label">World ID</label>
                <input class="input" id="moon-world-id" placeholder="e.g. deadbeef01">
              </div>
              <div class="field" style="flex:1">
                <label class="field-label">Seed ID <span class="text-dim">(optional)</span></label>
                <input class="input" id="moon-seed-id" placeholder="Leave blank for default seed">
              </div>
            </div>
            <button class="btn btn-primary mt-sm" onclick="SettingsRootsPage._orbit()">Orbit Moon</button>
          </div>
        </div>

        <!-- QR modal placeholder -->
        <div id="moon-qr-panel" style="display:none" class="card mt">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px">
            <span class="card-title" id="moon-qr-label">Moon QR Code</span>
            <button class="btn btn-ghost btn-sm" onclick="document.getElementById('moon-qr-panel').style.display='none'">✕</button>
          </div>
          <canvas id="moon-qr-canvas" width="200" height="200" style="display:block;margin:0 auto;border:1px solid var(--c-border);border-radius:4px"></canvas>
          <p class="text-sm text-dim mt-sm" style="text-align:center">Scan to share Moon World ID</p>
        </div>
      </div>`;
  }

  return {
    init() {
      document.getElementById('content').innerHTML =
        Utils.pageLoading();
      load();
    },

    _showQr(worldId) {
      const panel = document.getElementById('moon-qr-panel');
      const canvas = document.getElementById('moon-qr-canvas');
      const label  = document.getElementById('moon-qr-label');
      if (!panel || !canvas) return;
      if (label) label.textContent = `Moon QR — ${worldId}`;
      panel.style.display = 'block';
      // Render after display:block so canvas has dimensions
      requestAnimationFrame(() => QRCode.render(worldId, canvas, { size: 200 }));
    },

    async _orbit() {
      const worldId = document.getElementById('moon-world-id')?.value?.trim();
      const seed    = document.getElementById('moon-seed-id')?.value?.trim();
      if (!worldId) return Toast.error('World ID is required');
      try {
        await api.post(`/local/moons/${worldId}`, { seed: seed || undefined });
        Toast.success('Orbiting moon ' + worldId);
        load();
      } catch(e) { errToast(e); }
    },

    async _remove(id) {
      if (!await Modal.confirm(`Remove moon ${Utils.esc(id)}?`, { danger: true })) return;
      try {
        await api.delete(`/local/moons/${id}`);
        Toast.success('Moon removed');
        load();
      } catch(e) { errToast(e); }
    },
  };
})();
