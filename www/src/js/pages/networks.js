const NetworksPage = (() => {
  let _filter = 'all';
  let _nets   = [];       // enriched network objects
  let _nodeAddress = '';  // local ZT node address (10 hex)

  // ── Classify network type ─────────────────────────────────────────────────────
  // Central networks are in the local ZT list but also in Central API list.
  // We match by ID: if a local net ID exists in the Central set → CENTRAL.
  // If local net ID starts with node address → LOCAL CTRL.
  // Otherwise → JOINED (external network joined via ZT daemon).
  function _classify(localNets, centralNets, nodeAddr) {
    const centralIds = new Set(centralNets.map(n => n.id));
    return localNets.map(n => {
      let type = 'joined';
      if (centralIds.has(n.id))               type = 'central';
      else if (nodeAddr && n.id.startsWith(nodeAddr)) type = 'local_ctrl';
      return { ...n, _type: type };
    });
  }

  // ── Render ────────────────────────────────────────────────────────────────────
  function render(nets, filter) {
    const tabs = [
      { key: 'all',       label: 'All' },
      { key: 'joined',    label: 'Joined' },
      { key: 'central',   label: 'Central' },
      { key: 'local_ctrl',label: 'Local Ctrl' },
    ];
    const filtered = filter === 'all' ? nets : nets.filter(n => n._type === filter);

    const tabsHtml = tabs.map(t => {
      const cnt = t.key === 'all' ? nets.length : nets.filter(n => n._type === t.key).length;
      return `<div class="tab${filter === t.key ? ' active' : ''}" onclick="NetworksPage._setFilter('${t.key}')">
        ${t.label}${cnt ? ` <span class="tab-count">${cnt}</span>` : ''}
      </div>`;
    }).join('');

    const rows = filtered.map(n => {
      const typeBadge =
        n._type === 'central'    ? '<span class="badge badge-info">Central</span>' :
        n._type === 'local_ctrl' ? '<span class="badge badge-warn">Local Ctrl</span>' :
                                   '<span class="badge badge-muted">Joined</span>';
      const ips = (n.assignedAddresses || n.config?.assignedAddresses || []).join(', ') || '—';
      const status = n.status || '—';
      const statusClass = status === 'OK' ? 'text-success' : status === 'ACCESS_DENIED' ? 'text-danger' : '';
      return `<tr>
        <td>${typeBadge}</td>
        <td><span class="mono">${Utils.esc(n.id)}</span></td>
        <td>${Utils.esc(n.name || n.config?.name || '—')}</td>
        <td class="${statusClass}">${Utils.esc(status)}</td>
        <td class="text-sm">${Utils.esc(ips)}</td>
        <td>
          <div style="display:flex;gap:4px;flex-wrap:wrap">
            <button class="btn btn-sm btn-ghost" onclick="Router.navigate('/networks/${Utils.esc(n.id)}')">Details</button>
            <button class="btn btn-sm btn-danger" onclick="NetworksPage._leave('${Utils.esc(n.id)}')">Leave</button>
          </div>
        </td>
      </tr>`;
    }).join('');

    document.getElementById('content').innerHTML = `<div class="page">
      <div class="page-header">
        <h1 class="page-title">My Networks</h1>
        <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap">
          <input class="input" id="join-input" placeholder="Network ID (16 hex)" style="width:210px">
          <button class="btn btn-primary" onclick="NetworksPage._join()">+ Join Network</button>
        </div>
      </div>
      <div class="tabs mb">${tabsHtml}</div>
      ${!filtered.length
        ? `<div class="empty-state">
            <div class="empty-state-icon">🌐</div>
            <h3>${filter === 'all' ? 'No networks' : 'No ' + filter + ' networks'}</h3>
            <p>Enter a 16-character Network ID above and click Join.</p>
           </div>`
        : `<div class="table-wrap"><table>
            <thead><tr>
              <th>Type</th><th>Network ID</th><th>Name</th>
              <th>Status</th><th>Assigned IPs</th><th></th>
            </tr></thead>
            <tbody>${rows}</tbody>
          </table></div>`}
    </div>`;
  }

  // ── Load ──────────────────────────────────────────────────────────────────────
  async function load() {
    document.getElementById('content').innerHTML =
      Utils.pageLoading();

    // Fetch node address + local networks + central networks in parallel
    const [nodeRes, localRes, centralRes] = await Promise.allSettled([
      api.get('/local/status'),
      api.get('/local/networks'),
      api.get('/central/networks'),
    ]);

    _nodeAddress = nodeRes.status === 'fulfilled' ? (nodeRes.value?.address || '') : '';
    const localNets  = localRes.status === 'fulfilled'   ? (localRes.value  || []) : [];
    const centralNets = centralRes.status === 'fulfilled' ? (centralRes.value || []) : [];

    // Enrich: merge Central metadata into local network entries
    const centralMap = new Map(centralNets.map(n => [n.id, n]));
    const enriched = localNets.map(n => {
      const cn = centralMap.get(n.id);
      return { ...n, name: n.name || cn?.config?.name || '', _central: cn || null };
    });

    _nets = _classify(enriched, centralNets, _nodeAddress);
    State.set('networks', _nets);
    render(_nets, _filter);
  }

  // ── Public ────────────────────────────────────────────────────────────────────
  return {
    init() { _filter = 'all'; load(); },
    _setFilter(f) { _filter = f; render(_nets, _filter); },

    async _join() {
      const id = document.getElementById('join-input')?.value?.trim();
      if (!id || id.length !== 16) return Toast.error('Network ID must be 16 hex characters');
      try {
        await api.post(`/local/networks/${id}`, {});
        Toast.success('Joined ' + id);
        load();
      } catch(e) { errToast(e); }
    },

    async _leave(id) {
      if (!await Modal.confirm(`Leave network ${id}?`, { danger: true })) return;
      try {
        await api.delete(`/local/networks/${id}`);
        Toast.success('Left ' + id);
        load();
      } catch(e) { errToast(e); }
    },
  };
})();
