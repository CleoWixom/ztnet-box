const CtrlMembersPage = (() => {
  let _netId = null;
  let _members = [];
  let _src = 'local';

  function timeAgo(ms) {
    if (!ms) return '—';
    const s = Math.floor((Date.now() - ms) / 1000);
    if (s < 60) return s + 's ago';
    if (s < 3600) return Math.floor(s/60) + 'm ago';
    if (s < 86400) return Math.floor(s/3600) + 'h ago';
    return Math.floor(s/86400) + 'd ago';
  }

  function renderPanel(m) {
    return `<div class="panel-overlay active" id="member-panel" onclick="if(event.target.id==='member-panel')CtrlMembersPage._closePanel()">
      <div class="side-panel">
        <div class="panel-header">
          <div><div style="font-weight:600">${m.node_id||m.nodeId||m.address||'Member'}</div>
            <div class="text-sm text-dim">${m.name||''}</div></div>
          <button class="btn btn-ghost btn-icon" onclick="CtrlMembersPage._closePanel()">✕</button>
        </div>
        <div class="panel-body">
          <div class="toggle-wrap">
            <div><div class="toggle-label">Authorized</div></div>
            <label class="toggle">
              <input type="checkbox" ${m.authorized?'checked':''} onchange="CtrlMembersPage._updateMember('${m.node_id||m.nodeId}',{authorized:this.checked})">
              <div class="toggle-track"></div><div class="toggle-thumb"></div></label>
          </div>
          <div class="field mt">
            <label class="field-label">Name</label>
            <input class="input" id="m-name" value="${m.name||''}" placeholder="Display name">
          </div>
          <div class="field">
            <label class="field-label">Description</label>
            <textarea class="textarea" id="m-desc" rows="2">${m.description||''}</textarea>
          </div>
          <div class="field">
            <label class="field-label">Managed IPs</label>
            ${(m.ip_assignments||m.ipAssignments||[]).map(ip=>`<div style="display:flex;align-items:center;gap:4px;margin-bottom:4px">
              <span class="mono text-sm">${ip}</span>
              <button class="btn btn-ghost btn-icon btn-sm" onclick="CtrlMembersPage._removeIP('${m.node_id||m.nodeId}','${ip}')">✕</button>
            </div>`).join('')}
            <div class="input-row mt-sm">
              <input class="input" id="m-ip" placeholder="10.0.0.x">
              <button class="btn btn-ghost btn-sm" onclick="CtrlMembersPage._addIP('${m.node_id||m.nodeId}')">Add</button>
            </div>
          </div>
          <details class="mt"><summary style="cursor:pointer;color:var(--c-text-dim);font-size:var(--fs-sm)">Advanced</summary>
            <div class="mt-sm">
              <div class="toggle-wrap"><div class="toggle-label">Exclude from SSO</div>
                <label class="toggle"><input type="checkbox" ${m.ssoExempt||m.sso_exempt?'checked':''}
                  onchange="CtrlMembersPage._updateMember('${m.node_id||m.nodeId}',{sso_exempt:this.checked})">
                  <div class="toggle-track"></div><div class="toggle-thumb"></div></label></div>
              <div class="toggle-wrap"><div class="toggle-label">Allow Ethernet Bridging</div>
                <label class="toggle"><input type="checkbox" ${m.activeBridge||m.active_bridge?'checked':''}
                  onchange="CtrlMembersPage._updateMember('${m.node_id||m.nodeId}',{active_bridge:this.checked})">
                  <div class="toggle-track"></div><div class="toggle-thumb"></div></label></div>
              <div class="toggle-wrap"><div class="toggle-label">No Auto-Assign IPs</div>
                <label class="toggle"><input type="checkbox" ${m.noAutoAssignIps||m.no_auto_assign_ips?'checked':''}
                  onchange="CtrlMembersPage._updateMember('${m.node_id||m.nodeId}',{no_auto_assign_ips:this.checked})">
                  <div class="toggle-track"></div><div class="toggle-thumb"></div></label></div>
            </div>
          </details>
          <div class="detail-kv mt">
            <span class="k text-sm">Last Seen</span><span class="v text-sm">${timeAgo(m.lastSeen||m.last_online||m.lastOnline)}</span>
            <span class="k text-sm">Version</span><span class="v text-sm mono">${m.clientVersion||m.client_version||'—'}</span>
            <span class="k text-sm">Physical IP</span><span class="v text-sm mono">${m.physicalAddress||m.physical_address||'—'}</span>
          </div>
        </div>
        <div class="panel-footer">
          <button class="btn btn-ghost" onclick="CtrlMembersPage._save('${m.node_id||m.nodeId}')">Save</button>
          <button class="btn btn-danger" onclick="CtrlMembersPage._deleteMember('${m.node_id||m.nodeId}')">Delete</button>
        </div>
      </div>
    </div>`;
  }

  function renderTable() {
    if (!_members.length) {
      document.getElementById('members-body').innerHTML = `<tr><td colspan="7"><div class="empty-state"><div class="empty-state-icon">👥</div><h3>No members</h3></div></td></tr>`;
      return;
    }
    document.getElementById('members-body').innerHTML = _members.map(m => {
      const id = m.node_id||m.nodeId||m.address||'';
      const auth = m.authorized
        ? '<span class="badge badge-success">Authorized</span>'
        : '<span class="badge badge-muted">Unauthorized</span>';
      return `<tr style="cursor:pointer" onclick="CtrlMembersPage._openPanel('${Utils.esc(id)}')">
        <td>${auth}</td>
        <td><span class="mono">${Utils.esc(id)}</span></td>
        <td>${Utils.esc(m.name||'—')}</td>
        <td class="mono text-sm">${Utils.esc((m.ip_assignments||m.ipAssignments||[]).join(', ')||'—')}</td>
        <td class="text-sm">${timeAgo(m.lastOnline||m.last_online||m.lastSeen)}</td>
        <td class="mono text-sm">${Utils.esc(m.clientVersion||m.client_version||'—')}</td>
        <td class="mono text-sm">${Utils.esc(m.physicalAddress||m.physical_address||'—')}</td>
      </tr>`;
    }).join('');
  }

  async function load() {
    document.getElementById('members-body').innerHTML = `<tr><td colspan="7"><div class="loading-row"><div class="spinner"></div> Loading...</div></td></tr>`;
    try {
      if (_src === 'local') _members = await api.get(`/local/controller/networks/${_netId}/members`);
      else _members = await api.get(`/central/networks/${_netId}/members`);
    } catch(e) { _members = []; Toast.error(e.message); }
    renderTable();
  }

  return {
    init({ id }) {
      _netId = id; _src = 'local';
      document.getElementById('content').innerHTML = `<div class="page">
        <div class="page-header">
          <div>
            <button class="btn btn-ghost btn-sm mb-sm" onclick="Router.navigate('/controllers/networks')">← Networks</button>
            <h1 class="page-title">Members <span class="text-dim mono text-sm">${id}</span></h1>
          </div>
        </div>
        <div class="table-wrap"><table>
          <thead><tr><th>Auth</th><th>Address</th><th>Name</th><th>IPs</th><th>Last Seen</th><th>Version</th><th>Physical IP</th></tr></thead>
          <tbody id="members-body"></tbody>
        </table></div>
        <div id="member-panel-container"></div>
      </div>`;
      load();
    },
    _openPanel(id) {
      const m = _members.find(m => (m.node_id||m.nodeId) === id);
      if (!m) return;
      document.getElementById('member-panel-container').innerHTML = renderPanel(m);
    },
    _closePanel() { document.getElementById('member-panel-container').innerHTML = ''; },
    async _updateMember(id, update) {
      try {
        if (_src==='local') await api.put(`/local/controller/networks/${_netId}/members/${id}`, update);
        else await api.put(`/central/networks/${_netId}/members/${id}`, update);
        Toast.success('Updated'); load();
      } catch(e) { Toast.error(e.message); }
    },
    async _save(id) {
      const name = document.getElementById('m-name')?.value||'';
      const description = document.getElementById('m-desc')?.value||'';
      await this._updateMember(id, {name, description});
      this._closePanel();
    },
    async _addIP(id) {
      const ip = document.getElementById('m-ip')?.value?.trim();
      if (!ip) return;
      const m = _members.find(m=>(m.node_id||m.nodeId)===id);
      const ips = [...(m?.ip_assignments||m?.ipAssignments||[]), ip];
      await this._updateMember(id, {ip_assignments: ips});
    },
    async _removeIP(id, ip) {
      const m = _members.find(m=>(m.node_id||m.nodeId)===id);
      const ips = (m?.ip_assignments||m?.ipAssignments||[]).filter(i=>i!==ip);
      await this._updateMember(id, {ip_assignments: ips});
    },
    async _deleteMember(id) {
      if (!await Modal.confirm(`Delete member ${id}?`, {danger:true})) return;
      try {
        if (_src==='local') await api.delete(`/local/controller/networks/${_netId}/members/${id}`);
        else await api.delete(`/central/networks/${_netId}/members/${id}`);
        this._closePanel(); Toast.success('Deleted'); load();
      } catch(e) { Toast.error(e.message); }
    },
  };
})();
