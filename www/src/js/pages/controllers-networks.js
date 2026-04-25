const CtrlNetworksPage = (() => {
  let _nets = [];

  function render() {
    const rows = _nets.map(n => `<tr>
      <td><span class="badge ${n._src==='central'?'badge-info':'badge-primary'}">${n._src==='central'?'Central':'Local'}</span></td>
      <td><span class="mono">${Utils.esc(n.id)}</span></td>
      <td>${Utils.esc(n.name||n.config?.name||'—')}</td>
      <td>${n.totalMemberCount||n.member_count||'—'}</td>
      <td><div style="display:flex;gap:4px">
        <button class="btn btn-sm btn-ghost" onclick="Router.navigate('/controllers/members/${Utils.esc(n.id)}')">Members</button>
        <button class="btn btn-sm btn-ghost" onclick="Router.navigate('/controllers/config/${Utils.esc(n.id)}')">Config</button>
        <button class="btn btn-sm btn-danger" onclick="CtrlNetworksPage._delete('${Utils.esc(n.id)}','${Utils.esc(n._src)}')">Delete</button>
      </div></td>
    </tr>`).join('');

    document.getElementById('content').innerHTML = `<div class="page">
      <div class="page-header">
        <h1 class="page-title">Controller Networks</h1>
        <button class="btn btn-primary" onclick="CtrlNetworksPage._create()">+ New Network</button>
      </div>
      ${!_nets.length
        ? `<div class="empty-state"><div class="empty-state-icon">🖧</div><h3>No controller networks</h3><p>Create a network to manage members.</p></div>`
        : `<div class="table-wrap"><table><thead><tr><th>Type</th><th>Network ID</th><th>Name</th><th>Members</th><th></th></tr></thead>
           <tbody>${rows}</tbody></table></div>`}
    </div>`;
  }

  async function load() {
    _nets = [];
    try { _nets = (await api.get('/local/controller/networks')||[]).map(id=>({id, _src:'local', name:''})); }
    catch(e){}
    // Enrich local networks with details — all in parallel, not sequentially
    if (_nets.length) {
      const results = await Promise.allSettled(
        _nets.map(n => api.get(`/local/controller/networks/${n.id}`))
      );
      _nets = _nets.map((n, i) =>
        results[i].status === 'fulfilled' ? {...n, ...results[i].value, _src:'local'} : n
      );
    }
    try {
      const c = (await api.get('/central/networks'))||[];
      _nets = [..._nets, ...c.map(n=>({...n, name:n.config?.name||n.id, _src:'central'}))];
    } catch(e){}
    render();
  }

  return {
    init() { load(); },

    async _create() {
      const choice = await Modal.choice('New Network — choose controller', [
        {
          value: 'local',
          label: '🖥  ZT Local',
          description: 'Created on this device\'s built-in ZeroTier controller (zerotier-one). ' +
                       'No internet required. Ideal for private/offline networks.',
        },
        {
          value: 'central',
          label: '☁️  ZT Central',
          description: 'Created on ZeroTier Central (my.zerotier.com). ' +
                       'Requires a valid Central API token in Settings → Tokens.',
        },
      ]);
      if (!choice) return;
      try {
        if (choice === 'local') {
          await api.post('/local/controller/networks', { name: 'New Network', private: true });
        } else {
          await api.post('/central/networks', { config: { name: 'New Network', private: true } });
        }
        Toast.success('Network created');
        load();
      } catch(e) {
        if (e.actionHtml) {
          Toast.warn('⚠️ ' + e.actionHtml);
        } else {
          Toast.error(e.message);
        }
      }
    },

    async _delete(id, src) {
      if (!await Modal.confirm(`Delete network ${Utils.esc(id)}?<br><small>This cannot be undone.</small>`, { danger: true })) return;
      try {
        if (src === 'local') await api.delete(`/local/controller/networks/${id}`);
        else await api.delete(`/central/networks/${id}`);
        Toast.success('Deleted');
        load();
      } catch(e) { Toast.error(e.message); }
    },
  };
})();
