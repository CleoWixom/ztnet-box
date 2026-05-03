// UX-5: Merged Controllers page — Networks list + inline Members panel
// No separate Members page needed; members load in a side-panel on network row click.

const CtrlNetworksPage = (() => {
  let _nets = [];
  let _selectedNet = null;   // currently expanded network for members
  let _members = [];
  let _membersLoading = false;

  // ── Time helper ──────────────────────────────────────────────────────────────
  function timeAgo(ms) {
    if (!ms) return '—';
    const s = Math.floor((Date.now() - ms) / 1000);
    if (s < 60) return s + 's ago';
    if (s < 3600) return Math.floor(s/60) + 'm ago';
    if (s < 86400) return Math.floor(s/3600) + 'h ago';
    return Math.floor(s/86400) + 'd ago';
  }

  // ── Load networks ────────────────────────────────────────────────────────────
  async function loadNets() {
    document.getElementById('content').innerHTML =
      Utils.pageLoading();

    _nets = [];
    // Local controller networks
    try {
      const ids = await api.get('/local/controller/networks') || [];
      if (ids.length) {
        const details = await Promise.allSettled(ids.map(id => api.get(`/local/controller/networks/${id}`)));
        _nets = details.map((r, i) =>
          r.status === 'fulfilled' ? { ...r.value, id: r.value.id || ids[i], _src: 'local' }
                                   : { id: ids[i], name: '', _src: 'local' }
        );
      }
    } catch {}

    // Central networks
    try {
      const c = await api.get('/central/networks') || [];
      _nets = [..._nets, ...c.map(n => ({ ...n, name: n.config?.name || '', _src: 'central' }))];
    } catch {}

    renderPage();
  }

  // ── Load members for selected network ────────────────────────────────────────
  async function loadMembers(netId, src) {
    _membersLoading = true;
    _members = [];
    renderMembersPanel();
    try {
      if (src === 'local') _members = Object.values(await api.get(`/local/controller/networks/${netId}/members`) || {});
      else                  _members = await api.get(`/central/networks/${netId}/members`) || [];
    } catch {}
    _membersLoading = false;
    renderMembersPanel();
  }

  // ── Render members side panel ────────────────────────────────────────────────
  function renderMembersPanel() {
    const el = document.getElementById('ctrl-members-panel');
    if (!el || !_selectedNet) return;

    if (_membersLoading) {
      el.innerHTML = '<div class="loading-row"><div class="spinner"></div> Loading members…</div>';
      return;
    }

    const rows = _members.map(m => {
      const id = m.node_id || m.nodeId || m.address || '';
      const auth = m.authorized
        ? '<span class="badge badge-success">Auth</span>'
        : '<span class="badge badge-muted">Unauth</span>';
      const ips = (m.ip_assignments || m.ipAssignments || []).join(', ') || '—';
      const seen = timeAgo(m.lastOnline || m.last_online || m.lastSeen);
      return `<tr onclick="CtrlNetworksPage._openMemberPanel('${Utils.esc(id)}')" style="cursor:pointer">
        <td>${auth}</td>
        <td class="mono">${Utils.esc(id)}</td>
        <td>${Utils.esc(m.name || '—')}</td>
        <td class="mono text-sm">${Utils.esc(ips)}</td>
        <td class="text-sm">${seen}</td>
      </tr>`;
    }).join('');

    el.innerHTML = `
      <div class="section-title" style="display:flex;align-items:center;justify-content:space-between;margin-bottom:8px">
        <span>Members <span class="text-dim text-sm mono">${_selectedNet.id}</span></span>
        <button class="btn btn-ghost btn-sm" onclick="CtrlNetworksPage._closeMembers()">✕</button>
      </div>
      ${!_members.length
        ? '<div class="empty-state" style="padding:24px"><div class="empty-state-icon">👥</div><h3>No members</h3></div>'
        : `<div class="table-wrap"><table>
            <thead><tr><th>Auth</th><th>Address</th><th>Name</th><th>IPs</th><th>Last Seen</th></tr></thead>
            <tbody>${rows}</tbody>
          </table></div>`}
    `;
  }

  // ── Render full page ─────────────────────────────────────────────────────────
  function renderPage() {
    const rows = _nets.map(n => {
      const src = n._src;
      const badge = src === 'central'
        ? '<span class="badge badge-info">Central</span>'
        : '<span class="badge badge-primary">Local</span>';
      const isSelected = _selectedNet?.id === n.id;
      return `<tr class="${isSelected ? 'row-selected' : ''}" style="cursor:pointer"
              onclick="CtrlNetworksPage._selectNet('${Utils.esc(n.id)}','${src}')">
        <td>${badge}</td>
        <td class="mono">${Utils.esc(n.id)}</td>
        <td>${Utils.esc(n.name || n.config?.name || '—')}</td>
        <td>${n.totalMemberCount ?? n.member_count ?? '—'}</td>
        <td onclick="event.stopPropagation()">
          <div style="display:flex;gap:4px;flex-wrap:wrap">
            <button class="btn btn-sm btn-ghost"
              onclick="Router.navigate('/controllers/config/${Utils.esc(n.id)}')">Config</button>
            <button class="btn btn-sm btn-danger"
              onclick="CtrlNetworksPage._delete('${Utils.esc(n.id)}','${src}')">Delete</button>
          </div>
        </td>
      </tr>`;
    }).join('');

    document.getElementById('content').innerHTML = `<div class="page">
      <div class="page-header">
        <h1 class="page-title">Controllers</h1>
        <button class="btn btn-primary" onclick="CtrlNetworksPage._create()">+ New Network</button>
      </div>

      <div class="ctrl-split-layout">
        <div class="ctrl-nets-pane">
          ${!_nets.length
            ? `<div class="empty-state"><div class="empty-state-icon">🖧</div>
               <h3>No controller networks</h3>
               <p>Create a network to manage members.</p></div>`
            : `<div class="table-wrap"><table>
                 <thead><tr>
                   <th>Type</th><th>Network ID</th><th>Name</th><th>Members</th><th></th>
                 </tr></thead>
                 <tbody>${rows}</tbody>
               </table></div>`}
        </div>
        <div class="ctrl-members-pane" id="ctrl-members-panel">
          <div class="empty-state" style="padding:32px;opacity:.5">
            <div class="empty-state-icon">👈</div>
            <p>Click a network to view members</p>
          </div>
        </div>
      </div>
      <div id="member-edit-panel-container"></div>
    </div>`;

    if (_selectedNet) renderMembersPanel();
  }

  // ── Member edit side panel (reused from old CtrlMembersPage) ─────────────────
  function renderEditPanel(m) {
    const id = m.node_id || m.nodeId || m.address || '';
    return `<div class="panel-overlay active" id="member-edit-panel"
      onclick="if(event.target.id==='member-edit-panel')CtrlNetworksPage._closeEditPanel()">
      <div class="side-panel">
        <div class="panel-header">
          <div>
            <div style="font-weight:600">${Utils.esc(id)}</div>
            <div class="text-sm text-dim">${Utils.esc(m.name || '')}</div>
          </div>
          <button class="btn btn-ghost btn-icon" onclick="CtrlNetworksPage._closeEditPanel()">✕</button>
        </div>
        <div class="panel-body">
          <div class="toggle-wrap">
            <div><div class="toggle-label">Authorized</div></div>
            <label class="toggle">
              <input type="checkbox" ${m.authorized ? 'checked' : ''}
                onchange="CtrlNetworksPage._patchMember('${id}',{authorized:this.checked})">
              <div class="toggle-track"></div><div class="toggle-thumb"></div>
            </label>
          </div>
          <div class="field mt">
            <label class="field-label">Name</label>
            <input class="input" id="ep-name" value="${Utils.esc(m.name || '')}" placeholder="Display name">
          </div>
          <div class="field">
            <label class="field-label">Description</label>
            <textarea class="textarea" id="ep-desc" rows="2">${Utils.esc(m.description || '')}</textarea>
          </div>
          <div class="field">
            <label class="field-label">Managed IPs</label>
            ${(m.ip_assignments || m.ipAssignments || []).map(ip =>
              `<div style="display:flex;gap:4px;align-items:center;margin-bottom:4px">
                <span class="mono text-sm flex-1">${Utils.esc(ip)}</span>
                <button class="btn btn-ghost btn-icon btn-sm"
                  onclick="CtrlNetworksPage._removeIP('${id}','${ip}')">✕</button>
              </div>`
            ).join('')}
            <div class="input-row mt-sm">
              <input class="input" id="ep-ip" placeholder="10.0.0.x">
              <button class="btn btn-ghost btn-sm" onclick="CtrlNetworksPage._addIP('${id}')">Add</button>
            </div>
          </div>
          <details class="mt"><summary style="cursor:pointer;color:var(--c-text-dim);font-size:var(--fs-sm)">Advanced</summary>
            <div class="mt-sm">
              <div class="toggle-wrap"><div class="toggle-label">Allow Bridging</div>
                <label class="toggle"><input type="checkbox" ${m.activeBridge || m.active_bridge ? 'checked' : ''}
                  onchange="CtrlNetworksPage._patchMember('${id}',{active_bridge:this.checked})">
                  <div class="toggle-track"></div><div class="toggle-thumb"></div></label></div>
              <div class="toggle-wrap"><div class="toggle-label">No Auto-Assign IPs</div>
                <label class="toggle"><input type="checkbox" ${m.noAutoAssignIps || m.no_auto_assign_ips ? 'checked' : ''}
                  onchange="CtrlNetworksPage._patchMember('${id}',{no_auto_assign_ips:this.checked})">
                  <div class="toggle-track"></div><div class="toggle-thumb"></div></label></div>
            </div>
          </details>
          <div class="detail-kv mt">
            <span class="k text-sm">Last Seen</span>
            <span class="v text-sm">${timeAgo(m.lastOnline || m.last_online || m.lastSeen)}</span>
            <span class="k text-sm">Version</span>
            <span class="v text-sm mono">${Utils.esc(m.clientVersion || m.client_version || '—')}</span>
            <span class="k text-sm">Physical IP</span>
            <span class="v text-sm mono">${Utils.esc(m.physicalAddress || m.physical_address || '—')}</span>
          </div>
        </div>
        <div class="panel-footer">
          <button class="btn btn-ghost" onclick="CtrlNetworksPage._saveMember('${id}')">Save</button>
          <button class="btn btn-danger" onclick="CtrlNetworksPage._deleteMember('${id}')">Delete</button>
        </div>
      </div>
    </div>`;
  }

  // ── Public API ───────────────────────────────────────────────────────────────
  return {
    init() { _selectedNet = null; _members = []; loadNets(); },

    _selectNet(id, src) {
      const net = _nets.find(n => n.id === id);
      if (!net) return;
      _selectedNet = { id, src };
      renderPage();
      loadMembers(id, src);
    },

    _closeMembers() {
      _selectedNet = null;
      _members = [];
      renderPage();
    },

    _openMemberPanel(id) {
      const m = _members.find(m => (m.node_id || m.nodeId || m.address) === id);
      if (!m) return;
      document.getElementById('member-edit-panel-container').innerHTML = renderEditPanel(m);
    },

    _closeEditPanel() {
      const c = document.getElementById('member-edit-panel-container');
      if (c) c.innerHTML = '';
    },

    async _patchMember(id, update) {
      if (!_selectedNet) return;
      try {
        if (_selectedNet.src === 'local')
          await api.put(`/local/controller/networks/${_selectedNet.id}/members/${id}`, update);
        else
          await api.put(`/central/networks/${_selectedNet.id}/members/${id}`, { config: update });
        Toast.success('Updated');
        loadMembers(_selectedNet.id, _selectedNet.src);
      } catch(e) { Toast.error(e.message); }
    },

    async _saveMember(id) {
      const name = document.getElementById('ep-name')?.value || '';
      const description = document.getElementById('ep-desc')?.value || '';
      await this._patchMember(id, { name, description });
      this._closeEditPanel();
    },

    async _addIP(id) {
      const ip = document.getElementById('ep-ip')?.value?.trim();
      if (!ip) return;
      const m = _members.find(m => (m.node_id || m.nodeId) === id);
      const ips = [...(m?.ip_assignments || m?.ipAssignments || []), ip];
      await this._patchMember(id, { ip_assignments: ips });
    },

    async _removeIP(id, ip) {
      const m = _members.find(m => (m.node_id || m.nodeId) === id);
      const ips = (m?.ip_assignments || m?.ipAssignments || []).filter(i => i !== ip);
      await this._patchMember(id, { ip_assignments: ips });
    },

    async _deleteMember(id) {
      if (!await Modal.confirm(`Delete member ${id}?`, { danger: true })) return;
      try {
        if (_selectedNet.src === 'local')
          await api.delete(`/local/controller/networks/${_selectedNet.id}/members/${id}`);
        else
          await api.delete(`/central/networks/${_selectedNet.id}/members/${id}`);
        this._closeEditPanel();
        Toast.success('Deleted');
        loadMembers(_selectedNet.id, _selectedNet.src);
      } catch(e) { Toast.error(e.message); }
    },

    async _create() {
      const choice = await Modal.choice('New Network — choose controller', [
        { value: 'local', label: '🖥 ZT Local',
          description: 'Built-in controller on this device. No internet required.' },
        { value: 'central', label: '☁️ ZT Central',
          description: 'Requires a Central API token in Settings → Tokens.' },
      ]);
      if (!choice) return;
      try {
        if (choice === 'local')
          await api.post('/local/controller/networks', { name: 'New Network', private: true });
        else
          await api.post('/central/networks', { config: { name: 'New Network', private: true } });
        Toast.success('Network created');
        loadNets();
      } catch(e) { Toast.error(e.message); }
    },

    async _delete(id, src) {
      if (!await Modal.confirm(`Delete network ${id}?<br><small>This cannot be undone.</small>`, { danger: true })) return;
      try {
        if (src === 'local') await api.delete(`/local/controller/networks/${id}`);
        else await api.delete(`/central/networks/${id}`);
        if (_selectedNet?.id === id) { _selectedNet = null; _members = []; }
        Toast.success('Deleted');
        loadNets();
      } catch(e) { Toast.error(e.message); }
    },
  };
})();

// Alias: CtrlMembersPage is no longer a separate page; kept for router compatibility
const CtrlMembersPage = CtrlNetworksPage;
