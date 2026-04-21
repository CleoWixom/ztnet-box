const PeersPage = (() => {
  let _interval = null;

  async function load() {
    try {
      const peers = await api.get('/local/peers');
      State.set('peers', peers);
      render(peers);
    } catch(e) {
      document.getElementById('content').innerHTML =
        `<div class="page"><div class="banner banner-danger">❌ ${Utils.esc(e.message)}</div></div>`;
    }
  }

  async function init() {
    document.getElementById('content').innerHTML =
      '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading...</div></div>';
    await load();
    _interval = setInterval(load, 10000);
  }

  function render(peers) {
    const rows = (peers || []).map(p => {
      const latMs  = p.latency >= 0 ? p.latency : null;
      const latTxt = latMs !== null ? latMs + ' ms' : '—';
      const latCls = latMs !== null ? Utils.latencyClass(latMs) : '';
      return `<tr>
      <td class="mono">${Utils.esc(p.address)}</td>
      <td><span class="badge ${p.role==='PLANET'?'badge-info':p.role==='MOON'?'badge-warn':'badge-muted'}">${Utils.esc(p.role)}</span></td>
      <td class="${latCls}">${latTxt}</td>
      <td>${Utils.esc(p.version || '—')}</td>
      <td>${(p.paths || []).filter(path => path.active).length}</td>
    </tr>`;
    }).join('');

    document.getElementById('content').innerHTML = `<div class="page">
      <div class="page-header">
        <h1 class="page-title">Peers</h1>
        <span class="text-dim text-sm">${(peers||[]).length} peer${(peers||[]).length !== 1 ? 's' : ''}</span>
      </div>
      ${!rows
        ? `<div class="empty-state"><div class="empty-state-icon">🤝</div><h3>No peers</h3><p>Connect to a ZeroTier network to see peers.</p></div>`
        : `<div class="table-wrap"><table>
             <thead><tr>
               <th>Address</th><th>Role</th><th>Latency</th><th>Version</th><th>Active Paths</th>
             </tr></thead>
             <tbody>${rows}</tbody>
           </table></div>`}
    </div>`;
  }

  return {
    init,
    destroy() { if (_interval) { clearInterval(_interval); _interval = null; } },
  };
})();
