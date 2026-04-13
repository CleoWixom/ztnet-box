const PeersPage = (() => {
  // ── Init ──────────────────────────────────────────────────────────────────

  async function init() {
    document.getElementById('content').innerHTML =
      '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading...</div></div>';
    try {
      const peers = await api.get('/local/peers');
      State.set('peers', peers);
      render(peers);
    } catch(e) {
      document.getElementById('content').innerHTML =
        `<div class="page"><div class="banner banner-danger">❌ ${Utils.esc(e.message)}</div></div>`;
    }
  }

  // ── Render ────────────────────────────────────────────────────────────────

  function render(peers) {
    const rows = (peers || []).map(p => `<tr>
      <td class="mono">${Utils.esc(p.address)}</td>
      <td><span class="badge ${p.role==='PLANET'?'badge-info':p.role==='MOON'?'badge-warn':'badge-muted'}">${Utils.esc(p.role)}</span></td>
      <td>${p.latency >= 0 ? p.latency + 'ms' : '—'}</td>
      <td>${Utils.esc(p.version || '—')}</td>
      <td>${(p.paths || []).filter(path => path.active).length}</td>
    </tr>`).join('');

    document.getElementById('content').innerHTML = `<div class="page">
      <div class="page-header"><h1 class="page-title">Peers</h1></div>
      ${!rows
        ? `<div class="empty-state"><div class="empty-state-icon">🤝</div><h3>No peers</h3></div>`
        : `<div class="table-wrap"><table>
             <thead><tr>
               <th>Address</th><th>Role</th><th>Latency</th><th>Version</th><th>Active Paths</th>
             </tr></thead>
             <tbody>${rows}</tbody>
           </table></div>`}
    </div>`;
  }

  return { init };
})();
