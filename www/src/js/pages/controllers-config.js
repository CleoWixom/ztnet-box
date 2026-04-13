const CtrlConfigPage = (() => {
  const IP_POOLS = [
    '10.147.17.*','10.147.18.*','10.147.19.*','10.147.20.*',
    '10.144.*.*', '10.241.*.*', '10.242.*.*', '10.243.*.*',
    '10.244.*.*', '172.22.*.*', '172.23.*.*', '172.24.*.*',
    '172.25.*.*', '172.26.*.*', '172.27.*.*', '172.28.*.*',
    '172.29.*.*', '172.30.*.*', '192.168.191.*','192.168.192.*',
    '192.168.11.*','192.168.22.*','192.168.33.*','192.168.66.*',
  ];

  function poolToRange(p) {
    const parts = p.split('.');
    const base = parts.filter(x=>x!=='*').join('.');
    const stars = parts.filter(x=>x==='*').length;
    if (stars === 1) return { start: base+'.1', end: base+'.254' };
    if (stars === 2) return { start: base+'.0.1', end: base+'.255.254' };
    return { start: p, end: p };
  }

  let _net = null;
  let _id = null;
  let _src = 'local';
  let _selectedPool = null;

  function render() {
    const n = _net;
    const cfg = n?.config || n || {};
    document.getElementById('content').innerHTML = `<div class="page" style="max-width:720px">
      <div class="page-header">
        <div>
          <button class="btn btn-ghost btn-sm mb-sm" onclick="Router.navigate('/controllers/networks')">← Networks</button>
          <h1 class="page-title">Network Configuration</h1>
          <div class="text-dim mono text-sm">${_id}</div>
        </div>
        <button class="btn btn-primary" onclick="CtrlConfigPage._save()">Save Changes</button>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">Basics</div>
        <div class="field"><label class="field-label">Network ID</label>
          <input class="input" value="${_id}" readonly style="opacity:.6"></div>
        <div class="field"><label class="field-label">Name</label>
          <input class="input" id="cfg-name" value="${cfg.name||''}"></div>
        <div class="field"><label class="field-label">Description</label>
          <textarea class="textarea" id="cfg-desc">${cfg.description||n?.description||''}</textarea></div>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">Access Control</div>
        <div style="display:flex;gap:16px">
          <label style="display:flex;align-items:center;gap:6px;cursor:pointer">
            <input type="radio" name="access" value="private" ${cfg.private!==false?'checked':''} id="cfg-private"> Private
          </label>
          <label style="display:flex;align-items:center;gap:6px;cursor:pointer">
            <input type="radio" name="access" value="public" ${cfg.private===false?'checked':''} id="cfg-public"> Public
          </label>
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">IPv4 Auto-Assign</div>
        <div class="toggle-wrap mb">
          <div><div class="toggle-label">Enable IPv4 Auto-Assignment</div></div>
          <label class="toggle"><input type="checkbox" id="cfg-v4" ${cfg.v4AssignMode?.zt||cfg.v4_assign_mode?.zt?'checked':''}>
            <div class="toggle-track"></div><div class="toggle-thumb"></div></label>
        </div>
        <div class="ip-pool-grid">
          ${IP_POOLS.map(p=>`<button class="ip-pool-btn${_selectedPool===p?' selected':''}" onclick="CtrlConfigPage._selectPool('${p}')">${p}</button>`).join('')}
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">IPv6 Auto-Assign</div>
        <div class="toggle-wrap">
          <div><div class="toggle-label">RFC4193 (/128 per node)</div></div>
          <label class="toggle"><input type="checkbox" id="cfg-rfc4193" ${cfg.v6AssignMode?.rfc4193||cfg.v6_assign_mode?.rfc4193?'checked':''}>
            <div class="toggle-track"></div><div class="toggle-thumb"></div></label>
        </div>
        <div class="toggle-wrap">
          <div><div class="toggle-label">6PLANE (/80 prefix)</div></div>
          <label class="toggle"><input type="checkbox" id="cfg-6plane" ${cfg.v6AssignMode?.['6plane']||cfg.v6_assign_mode?.['6plane']?'checked':''}>
            <div class="toggle-track"></div><div class="toggle-thumb"></div></label>
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">Multicast</div>
        <div class="field"><label class="field-label">Multicast Recipient Limit</label>
          <input class="input" id="cfg-multicast" type="number" value="${cfg.multicastLimit||cfg.multicast_limit||32}" style="width:120px"></div>
        <div class="toggle-wrap">
          <div><div class="toggle-label">Enable Broadcast</div></div>
          <label class="toggle"><input type="checkbox" id="cfg-broadcast" ${cfg.enableBroadcast||cfg.enable_broadcast?'checked':''}>
            <div class="toggle-track"></div><div class="toggle-thumb"></div></label>
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">DNS</div>
        <div class="field"><label class="field-label">Search Domain</label>
          <input class="input" id="cfg-dns-domain" value="${cfg.dns?.domain||''}"></div>
        <div class="field"><label class="field-label">DNS Servers</label>
          <div id="cfg-dns-servers">${(cfg.dns?.servers||[]).map(s=>`
            <div style="display:flex;gap:4px;margin-bottom:4px">
              <span class="mono text-sm" style="flex:1;padding:6px 8px;background:var(--c-bg);border-radius:4px">${s}</span>
              <button class="btn btn-ghost btn-icon btn-sm" onclick="this.closest('div').remove()">✕</button>
            </div>`).join('')}</div>
          <div class="input-row mt-sm">
            <input class="input" id="cfg-dns-add" placeholder="8.8.8.8">
            <button class="btn btn-ghost btn-sm" onclick="CtrlConfigPage._addDNS()">Add</button>
          </div>
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">Manually Add Member</div>
        <div class="input-row">
          <input class="input" id="cfg-add-member" placeholder="10-character node ID" maxlength="10">
          <button class="btn btn-ghost" onclick="CtrlConfigPage._addMember()">Add Member</button>
        </div>
      </div>

      <div class="mt" style="border-top:1px solid var(--c-border);padding-top:var(--gap)">
        <button class="btn btn-danger" onclick="CtrlConfigPage._delete()">Delete Network</button>
      </div>
    </div>`;
  }

  return {
    async init({ id }) {
      _id = id; _src = 'local';
      document.getElementById('content').innerHTML = '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading...</div></div>';
      try {
        _net = await api.get(`/local/controller/networks/${id}`);
      } catch(e) {
        try { _net = await api.get(`/central/networks/${id}`); _src = 'central'; }
        catch(e2) { Toast.error(e2.message); }
      }
      render();
    },
    _selectPool(p) { _selectedPool = p; render(); },
    _addDNS() {
      const val = document.getElementById('cfg-dns-add')?.value?.trim();
      if (!val) return;
      const container = document.getElementById('cfg-dns-servers');
      container.insertAdjacentHTML('beforeend', `<div style="display:flex;gap:4px;margin-bottom:4px">
        <span class="mono text-sm" style="flex:1;padding:6px 8px;background:var(--c-bg);border-radius:4px">${val}</span>
        <button class="btn btn-ghost btn-icon btn-sm" onclick="this.closest('div').remove()">✕</button>
      </div>`);
      document.getElementById('cfg-dns-add').value = '';
    },
    async _addMember() {
      const nodeId = document.getElementById('cfg-add-member')?.value?.trim();
      if (!nodeId || nodeId.length !== 10) return Toast.error('Node ID must be 10 hex characters');
      try {
        if (_src==='local') await api.put(`/local/controller/networks/${_id}/members/${nodeId}`, {authorized:false});
        else await api.put(`/central/networks/${_id}/members/${nodeId}`, {authorized:false});
        Toast.success(`Member ${nodeId} added`);
      } catch(e) { Toast.error(e.message); }
    },
    async _save() {
      const name = document.getElementById('cfg-name')?.value||'';
      const description = document.getElementById('cfg-desc')?.value||'';
      const private_ = document.getElementById('cfg-private')?.checked;
      const v4 = document.getElementById('cfg-v4')?.checked;
      const rfc4193 = document.getElementById('cfg-rfc4193')?.checked;
      const plan6 = document.getElementById('cfg-6plane')?.checked;
      const multicastLimit = parseInt(document.getElementById('cfg-multicast')?.value)||32;
      const enableBroadcast = document.getElementById('cfg-broadcast')?.checked;
      const dnsDomain = document.getElementById('cfg-dns-domain')?.value?.trim();
      const dnsServers = [...document.querySelectorAll('#cfg-dns-servers .mono')].map(el=>el.textContent.trim());
      const ipRange = _selectedPool ? poolToRange(_selectedPool) : null;

      const body = _src === 'central' ? {
        config: {
          name, private: private_, multicastLimit, enableBroadcast,
          v4AssignMode: { zt: v4 },
          v6AssignMode: { rfc4193, '6plane': plan6 },
          ipAssignmentPools: ipRange ? [ipRange] : undefined,
          dns: dnsDomain ? { domain: dnsDomain, servers: dnsServers } : undefined,
        }
      } : {
        name, private: private_, multicastLimit, enableBroadcast,
        v4AssignMode: { zt: v4 },
        v6AssignMode: { rfc4193, plan6 },
        ipAssignmentPools: ipRange ? [ipRange] : undefined,
        dns: dnsDomain ? { domain: dnsDomain, servers: dnsServers } : undefined,
      };

      try {
        if (_src==='local') await api.put(`/local/controller/networks/${_id}`, body);
        else await api.put(`/central/networks/${_id}`, body);
        Toast.success('Saved');
      } catch(e) { Toast.error(e.message); }
    },
    async _delete() {
      if (!await Modal.confirm('Delete this network and all its members? This cannot be undone.', {danger:true})) return;
      try {
        if (_src==='local') await api.delete(`/local/controller/networks/${_id}`);
        else await api.delete(`/central/networks/${_id}`);
        Toast.success('Deleted'); Router.navigate('/controllers/networks');
      } catch(e) { Toast.error(e.message); }
    },
  };
})();
