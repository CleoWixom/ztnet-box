const NetworksPage = (() => {
  function render(nets, filter) {
    const tabs = ['all','own','central'];
    const labels = { all:'All', own:'My Networks', central:'Central' };
    const filtered = filter === 'all' ? nets : nets.filter(n => n._src === filter);
    const tabsHtml = tabs.map(t => `<div class="tab${filter===t?' active':''}" onclick="NetworksPage._setFilter('${t}')">${labels[t]}</div>`).join('');

    const rows = filtered.map(n => {
      const typeHtml = n._src === 'central'
        ? '<span class="badge badge-info">Central</span>'
        : '<span class="badge badge-primary">Local</span>';
      return `<tr>
        <td>${typeHtml}</td>
        <td><span class="mono net-id">${n.id}</span></td>
        <td>${n.name || '<span class="text-mute">—</span>'}</td>
        <td>${n.status || '—'}</td>
        <td>${(n.assignedAddresses||n.config?.assignedAddresses||[]).join(', ') || '—'}</td>
        <td>
          <div style="display:flex;gap:4px">
            <button class="btn btn-sm btn-ghost" onclick="Router.navigate('/networks/${n.id}')">Details</button>
            <button class="btn btn-sm btn-danger" onclick="NetworksPage._leave('${n.id}')">Leave</button>
          </div>
        </td>
      </tr>`;
    }).join('');

    document.getElementById('content').innerHTML = `<div class="page">
      <div class="page-header">
        <h1 class="page-title">My Networks</h1>
        <div style="display:flex;gap:8px">
          <input class="input" id="join-input" placeholder="Network ID (16 hex chars)" style="width:220px">
          <button class="btn btn-primary" onclick="NetworksPage._join()">Join Network</button>
        </div>
      </div>
      <div class="tabs">${tabsHtml}</div>
      ${!filtered.length
        ? `<div class="empty-state"><div class="empty-state-icon">🌐</div><h3>No networks</h3><p>Join a network to get started.</p></div>`
        : `<div class="table-wrap"><table>
            <thead><tr><th>Type</th><th>Network ID</th><th>Name</th><th>Status</th><th>IPs</th><th></th></tr></thead>
            <tbody>${rows}</tbody></table></div>`}
    </div>`;
  }

  let _filter = 'all';
  let _nets = [];

  async function load() {
    try {
      const local = await api.get('/local/networks');
      _nets = local.map(n => ({...n, _src:'own'}));
    } catch(e) { _nets = []; }
    try {
      const central = await api.get('/central/networks');
      _nets = [..._nets, ...central.map(n => ({id: n.id, name: n.config?.name||n.id, _src:'central', ...n}))];
    } catch(e) {}
    State.set('networks', _nets);
    render(_nets, _filter);
  }

  return {
    init() { load(); },
    _setFilter(f) { _filter = f; render(_nets, _filter); },
    async _join() {
      const id = document.getElementById('join-input')?.value?.trim();
      if (!id || id.length !== 16) return Toast.error('Network ID must be 16 hex characters');
      try { await api.post(`/local/networks/${id}`, {}); Toast.success('Joined ' + id); load(); }
      catch(e) { Toast.error(e.message); }
    },
    async _leave(id) {
      if (!await Modal.confirm(`Leave network ${id}?`, {danger:true})) return;
      try { await api.delete(`/local/networks/${id}`); Toast.success('Left ' + id); load(); }
      catch(e) { Toast.error(e.message); }
    },
  };
})();
