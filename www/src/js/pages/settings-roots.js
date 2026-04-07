const SettingsRootsPage = (() => {
  return {
    async init() {
      document.getElementById('content').innerHTML = '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading...</div></div>';
      let moons = [], peers = [];
      try { moons = await api.get('/local/moons'); } catch(e){}
      try { peers = await api.get('/local/peers'); } catch(e){}
      const moonPeers = peers.filter(p => p.role === 'MOON');
      const rows = moons.map(m => `<tr>
        <td class="mono">${m.id}</td>
        <td class="text-sm">${new Date(m.timestamp).toLocaleDateString()}</td>
        <td class="text-sm">${(m.roots||[]).map(r=>r.identity?.slice(0,20)+'...').join('<br>')}</td>
        <td><button class="btn btn-danger btn-sm" onclick="SettingsRootsPage._remove('${m.id}')">Remove</button></td>
      </tr>`).join('');

      document.getElementById('content').innerHTML = `<div class="page">
        <div class="page-header">
          <h1 class="page-title">Root Servers (Moons)</h1>
        </div>
        ${moonPeers.length?`<div class="banner banner-info">🌙 ${moonPeers.length} moon peer${moonPeers.length!==1?'s':''} connected: ${moonPeers.map(p=>p.address).join(', ')}</div>`:''}
        ${!moons.length
          ? `<div class="empty-state"><div class="empty-state-icon">🌙</div><h3>No moons configured</h3>
             <p>Moons let you use your own root servers instead of or in addition to the default ZeroTier planets.</p>
             <a href="https://docs.zerotier.com/zerotier/moons" target="_blank" class="btn btn-ghost mt">Documentation ↗</a></div>`
          : `<div class="table-wrap"><table><thead><tr><th>World ID</th><th>Timestamp</th><th>Roots</th><th></th></tr></thead>
             <tbody>${rows}</tbody></table></div>`}
        <div class="card mt">
          <div class="card-title mb">Orbit a Moon</div>
          <div class="input-row">
            <input class="input" id="moon-world-id" placeholder="World ID">
            <input class="input" id="moon-seed-id" placeholder="Seed ID (optional)">
            <button class="btn btn-primary" onclick="SettingsRootsPage._orbit()">Orbit</button>
          </div>
        </div>
      </div>`;
    },
    async _orbit() {
      const worldId = document.getElementById('moon-world-id')?.value?.trim();
      const seed    = document.getElementById('moon-seed-id')?.value?.trim();
      if (!worldId) return Toast.error('World ID is required');
      try { await api.post(`/local/moons/${worldId}`, { seed: seed || undefined }); Toast.success('Orbiting moon ' + worldId); this.init(); }
      catch(e) { Toast.error(e.message); }
    },
    async _remove(id) {
      if (!await Modal.confirm(`Remove moon ${id}?`, {danger:true})) return;
      try { await api.delete(`/local/moons/${id}`); Toast.success('Moon removed'); this.init(); }
      catch(e) { Toast.error(e.message); }
    },
  };
})();
