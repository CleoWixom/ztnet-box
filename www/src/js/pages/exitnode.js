const ExitnodePage = (() => {
  let _data = {};

  async function load() {
    const [platform, deps, ifaces, status] = await Promise.allSettled([
      api.get('/exitnode/platform'),
      api.get('/exitnode/deps'),
      api.get('/exitnode/interfaces'),
      api.get('/exitnode/status'),
    ]);
    _data = {
      platform: platform.value,
      deps:     deps.value,
      ifaces:   ifaces.value||[],
      status:   status.value,
    };
    render();
  }

  function render() {
    const { platform, deps, ifaces, status } = _data;
    const el = document.getElementById('content');

    if (!platform?.supported) {
      el.innerHTML = `<div class="page"><div class="page-header"><h1 class="page-title">Exit Node</h1></div>
        <div class="banner banner-warn">⚠️ Not supported on this platform: ${platform?.reason||platform?.os||'unknown'}</div></div>`;
      return;
    }

    const depsList = deps ? `
      <div class="step-item${deps.iptables||deps.nftables?' done':''}">
        <div class="step-num">${deps.iptables||deps.nftables?'✓':'1'}</div>
        <div class="step-content">
          <div class="step-title">Firewall Tools</div>
          <div class="text-sm">${deps.nftables?'nftables ✓':''} ${deps.iptables?'iptables ✓':''}</div>
          ${(!deps.iptables&&!deps.nftables)?`<button class="btn btn-primary btn-sm mt-sm" onclick="ExitnodePage._installDeps()">Install missing</button>`:''}
        </div>
      </div>
      <div class="step-item${deps.is_root?' done':''}">
        <div class="step-num">${deps.is_root?'✓':'2'}</div>
        <div class="step-content">
          <div class="step-title">Root Access</div>
          <div class="text-sm">${deps.is_root?'Running as root':'Restart with sudo for firewall management'}</div>
        </div>
      </div>` : '<div class="loading-row"><div class="spinner"></div> Checking dependencies...</div>';

    const nets = State.get('networks')||[];
    const netOpts = nets.filter(n=>n.status==='OK'||n._src).map(n=>`<option value="${n.id}">${n.id} ${n.name?('('+n.name+')'):''}  </option>`).join('');

    const wanIfaces = (ifaces||[]).filter(i=>!i.is_zerotier&&i.name!=='lo');
    const wanOpts = wanIfaces.map(i=>`<option value="${i.name}">${i.name} ${i.addresses?.[0]?('— '+i.addresses[0]):''}  </option>`).join('');

    const statusBlock = status ? `
      <div class="card mt">
        <div class="card-header"><div class="card-title">Status</div>
          <span class="badge ${status.enabled?'badge-success':'badge-muted'}">${status.enabled?'Enabled':'Disabled'}</span>
        </div>
        ${status.enabled?`
          <div class="detail-kv">
            <span class="k">ZT Network</span><span class="v mono">${status.zt_network_id||'—'}</span>
            <span class="k">WAN Interface</span><span class="v mono">${status.wan_interface||'—'}</span>
            <span class="k">Backend</span><span class="v">${status.backend||'—'}</span>
            <span class="k">Since</span><span class="v">${status.applied_at?new Date(status.applied_at).toLocaleString():'—'}</span>
          </div>`:'<div class="text-dim text-sm">Exit Node is not active.</div>'}
      </div>` : '';

    el.innerHTML = `<div class="page">
      <div class="page-header"><h1 class="page-title">Exit Node</h1></div>
      <div class="section"><div class="section-title">Dependencies</div>
        <div class="card"><div class="step-list">${depsList}</div></div>
      </div>
      <div class="section"><div class="section-title">Configuration</div>
        <div class="card">
          <div class="field"><label class="field-label">ZeroTier Network</label>
            <select class="select" id="en-net">${netOpts||'<option value="">No networks available</option>'}</select></div>
          <div class="field"><label class="field-label">WAN Interface</label>
            <select class="select" id="en-wan">${wanOpts||'<option value="">No interfaces detected</option>'}</select></div>
          ${status?.enabled
            ? `<button class="btn btn-danger" onclick="ExitnodePage._disable()">Disable Exit Node</button>`
            : `<button class="btn btn-primary" onclick="ExitnodePage._enable()">Enable Exit Node</button>`}
        </div>
      </div>
      ${statusBlock}
    </div>`;
  }

  return {
    async init() {
      document.getElementById('content').innerHTML = '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading...</div></div>';
      // Load networks first for the dropdown
      try { const nets = await api.get('/local/networks'); State.set('networks', nets); } catch(e){}
      await load();
    },
    async _installDeps() {
      try { await api.post('/exitnode/deps/install'); Toast.success('Dependencies installed'); load(); }
      catch(e) { Toast.error(e.message); }
    },
    async _enable() {
      const zt = document.getElementById('en-net')?.value;
      const wan = document.getElementById('en-wan')?.value;
      if (!zt) return Toast.error('Select a ZeroTier network');
      if (!wan) return Toast.error('Select a WAN interface');
      if (!await Modal.confirm(`Enable Exit Node?<br><small>All traffic on <b>${zt}</b> will route through <b>${wan}</b>. Ensure this is intentional.</small>`)) return;
      try { await api.post('/exitnode/enable', { zt_interface: zt, wan_interface: wan }); Toast.success('Exit Node enabled'); load(); }
      catch(e) { Toast.error(e.message); }
    },
    async _disable() {
      if (!await Modal.confirm('Disable Exit Node?', {danger:true})) return;
      try { await api.post('/exitnode/disable'); Toast.success('Exit Node disabled'); load(); }
      catch(e) { Toast.error(e.message); }
    },
  };
})();
