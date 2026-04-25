const NetworkDetailPage = (() => {
  function renderDetail(n, tab) {
    const tabs = ['details','config'];
    const tabsHtml = tabs.map(t => `<div class="tab${tab===t?' active':''}" onclick="NetworkDetailPage._tab('${n.id}','${t}')">${t.charAt(0).toUpperCase()+t.slice(1)}</div>`).join('');
    const detailHtml = tab === 'details' ? `
      <div class="detail-kv mt">
        <span class="k">Network ID</span><span class="v mono">${n.id}</span>
        <span class="k">Name</span><span class="v">${n.name||'—'}</span>
        <span class="k">Status</span><span class="v">${n.status||'—'}</span>
        <span class="k">Type</span><span class="v">${n.type_||n.type||'—'}</span>
        <span class="k">MAC</span><span class="v mono">${n.mac||'—'}</span>
        <span class="k">MTU</span><span class="v">${n.mtu||'—'}</span>
        <span class="k">Broadcast</span><span class="v">${n.broadcastEnabled ? 'Enabled' : 'Disabled'}</span>
        <span class="k">Bridge</span><span class="v">${n.bridge ? 'Yes' : 'No'}</span>
        <span class="k">Managed IPs</span><span class="v">${(n.assignedAddresses||[]).join(', ')||'—'}</span>
        <span class="k">DNS</span><span class="v">${n.dns ? n.dns.domain + ' (' + (n.dns.servers||[]).join(', ') + ')' : 'Not configured'}</span>
      </div>
      <div class="mt" style="display:flex;gap:8px;align-items:center">
        <button class="btn btn-ghost btn-sm" onclick="navigator.clipboard.writeText('${n.id}').then(()=>Toast.success('Copied!'))">📋 Copy ID</button>
      </div>
      <div class="qr-wrap mt"><canvas id="qr-canvas" width="200" height="200"></canvas></div>` : `
      <div class="mt">
        <div class="toggle-wrap">
          <div><div class="toggle-label">Route all traffic through ZeroTier</div>
            <div class="toggle-hint text-sm">Requires external default route configuration on the exit node</div></div>
          <label class="toggle">
            <input type="checkbox" id="allow-default" ${n.allowDefault?'checked':''}
              onchange="NetworkDetailPage._saveConfig('${n.id}',{allowDefault:this.checked})">
            <div class="toggle-track"></div><div class="toggle-thumb"></div>
          </label>
        </div>
        <div class="toggle-wrap">
          <div><div class="toggle-label">Allow managed routes</div></div>
          <label class="toggle">
            <input type="checkbox" id="allow-managed" ${n.allowManaged!==false?'checked':''}
              onchange="NetworkDetailPage._saveConfig('${n.id}',{allowManaged:this.checked})">
            <div class="toggle-track"></div><div class="toggle-thumb"></div>
          </label>
        </div>
        <div class="toggle-wrap">
          <div><div class="toggle-label">Allow global IPs</div></div>
          <label class="toggle">
            <input type="checkbox" id="allow-global" ${n.allowGlobal?'checked':''}
              onchange="NetworkDetailPage._saveConfig('${n.id}',{allowGlobal:this.checked})">
            <div class="toggle-track"></div><div class="toggle-thumb"></div>
          </label>
        </div>
        <div class="toggle-wrap">
          <div><div class="toggle-label">Allow DNS</div></div>
          <label class="toggle">
            <input type="checkbox" id="allow-dns" ${n.allowDns?'checked':''}
              onchange="NetworkDetailPage._saveConfig('${n.id}',{allowDns:this.checked})">
            <div class="toggle-track"></div><div class="toggle-thumb"></div>
          </label>
        </div>
      </div>`;

    document.getElementById('content').innerHTML = `<div class="page">
      <div class="page-header">
        <div>
          <button class="btn btn-ghost btn-sm mb-sm" onclick="Router.navigate('/networks')">← Back</button>
          <h1 class="page-title mono">${n.id}</h1>
          <div class="text-dim text-sm">${n.name||''}</div>
        </div>
        <button class="btn btn-danger btn-sm" onclick="NetworkDetailPage._leave('${n.id}')">Leave Network</button>
      </div>
      <div class="tabs">${tabsHtml}</div>
      ${detailHtml}
    </div>`;

    if (tab === 'details') {
      const canvas = document.getElementById('qr-canvas');
      if (canvas) QRCode.render(n.id, canvas);
    }
  }

  return {
    async init({ id }) {
      document.getElementById('content').innerHTML = '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading...</div></div>';
      try {
        const n = await api.get(`/local/networks/${id}`);
        renderDetail(n, 'details');
      } catch(e) {
        document.getElementById('content').innerHTML = `<div class="page"><div class="banner banner-danger">❌ ${e.message}</div></div>`;
      }
    },
    async _tab(id, tab) {
      try { const n = await api.get(`/local/networks/${id}`); renderDetail(n, tab); }
      catch(e) { errToast(e); }
    },
    async _saveConfig(id, update) {
      try { await api.post(`/local/networks/${id}`, update); Toast.success('Saved'); }
      catch(e) { errToast(e); }
    },
    async _leave(id) {
      if (!await Modal.confirm(`Leave network ${id}?`, {danger:true})) return;
      try { await api.delete(`/local/networks/${id}`); Toast.success('Left'); Router.navigate('/networks'); }
      catch(e) { errToast(e); }
    },
  };
})();
