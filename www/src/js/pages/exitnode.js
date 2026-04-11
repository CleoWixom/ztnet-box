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
          <div class="step-title">Firewall Tools (IPv4)</div>
          <div class="text-sm">${deps.nftables?'nftables ✓':''} ${deps.iptables?'iptables ✓':''}</div>
          ${(!deps.iptables&&!deps.nftables)?`<button class="btn btn-primary btn-sm mt-sm" onclick="ExitnodePage._installDeps()">Install missing</button>`:''}
        </div>
      </div>
      <div class="step-item${deps.ip6tables?' done':''}">
        <div class="step-num">${deps.ip6tables?'✓':'2'}</div>
        <div class="step-content">
          <div class="step-title">ip6tables (IPv6)</div>
          <div class="text-sm">${deps.ip6tables?'ip6tables ✓':'Not found — IPv6 Exit Node unavailable'}</div>
        </div>
      </div>
      <div class="step-item${deps.is_root?' done':''}">
        <div class="step-num">${deps.is_root?'✓':'3'}</div>
        <div class="step-content">
          <div class="step-title">Root Access</div>
          <div class="text-sm">${deps.is_root?'Running as root':'Restart with sudo for firewall management'}</div>
        </div>
      </div>` : '<div class="loading-row"><div class="spinner"></div> Checking dependencies...</div>';

    const nets = State.get('networks')||[];
    const netOpts = nets.filter(n=>n.status==='OK'||n._src).map(n=>`<option value="${n.id}">${n.id} ${n.name?('('+n.name+')'):''}  </option>`).join('');

    const wanIfaces = (ifaces||[]).filter(i=>!i.is_zerotier&&i.name!=='lo');
    const wanOpts = wanIfaces.map(i=>`<option value="${i.name}">${i.name} ${i.addresses?.[0]?('— '+i.addresses[0]):''}  </option>`).join('');

    const ipv6Section = `
      <div class="field mt">
        <label class="field-label" style="display:flex;align-items:center;gap:0.5rem;">
          <input type="checkbox" id="en-ipv6" onchange="ExitnodePage._toggleIpv6()" ${status?.enable_ipv6?'checked':''}>
          Enable IPv6 (ip6tables)
        </label>
        <div class="text-dim text-sm">Adds stateful ip6tables FORWARD rules and IPv6 MASQUERADE on WAN. Clients must have allowGlobal=1.</div>
      </div>
      <div id="ipv6-prefix-row" class="field" style="display:${status?.enable_ipv6?'block':'none'}">
        <label class="field-label">IPv6 Prefix <span class="text-dim">(optional)</span></label>
        <input class="input" id="en-ipv6-prefix" placeholder="e.g. 2001:db8::/64" value="${status?.ipv6_prefix||''}">
        <div class="text-dim text-sm">Scope FORWARD rules to this prefix. Leave empty for wildcard (all ZT IPv6 traffic).</div>
      </div>`;

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
            <span class="k">IPv6</span><span class="v">${status.enable_ipv6?('✓ '+(status.ipv6_prefix||'(all prefixes)')):'Disabled'}</span>
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
          ${ipv6Section}
          ${status?.enabled
            ? `<button class="btn btn-danger mt" onclick="ExitnodePage._disable()">Disable Exit Node</button>`
            : `<button class="btn btn-primary mt" onclick="ExitnodePage._enable()">Enable Exit Node</button>`}
        </div>
      </div>
      ${statusBlock}
    </div>`;
  }

  return {
    async init() {
      document.getElementById('content').innerHTML = '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading...</div></div>';
      try { const nets = await api.get('/local/networks'); State.set('networks', nets); } catch(e){}
      await load();
    },
    _toggleIpv6() {
      const checked = document.getElementById('en-ipv6')?.checked;
      const row = document.getElementById('ipv6-prefix-row');
      if (row) row.style.display = checked ? 'block' : 'none';
    },
    async _installDeps() {
      try { await api.post('/exitnode/deps/install'); Toast.success('Dependencies installed'); load(); }
      catch(e) { Toast.error(e.message); }
    },
    async _enable() {
      const zt  = document.getElementById('en-net')?.value;
      const wan = document.getElementById('en-wan')?.value;
      const ipv6 = document.getElementById('en-ipv6')?.checked || false;
      const ipv6_prefix = document.getElementById('en-ipv6-prefix')?.value.trim() || null;
      if (!zt)  return Toast.error('Select a ZeroTier network');
      if (!wan) return Toast.error('Select a WAN interface');
      const ipv6note = ipv6 ? '<br><small>IPv6 ip6tables rules will also be applied.</small>' : '';
      if (!await Modal.confirm(`Enable Exit Node?<br><small>All traffic on <b>${zt}</b> will route through <b>${wan}</b>.</small>${ipv6note}`)) return;
      try {
        const res = await api.post('/exitnode/enable', {
          zt_interface: zt,
          wan_interface: wan,
          network_id: zt,
          enable_ipv6: ipv6,
          ipv6_prefix,
        });
        if (res.warnings?.length) res.warnings.forEach(w => Toast.warn(w));
        Toast.success('Exit Node enabled');
        load();
      } catch(e) { Toast.error(e.message); }
    },
    async _disable() {
      if (!await Modal.confirm('Disable Exit Node?', {danger:true})) return;
      try { await api.post('/exitnode/disable'); Toast.success('Exit Node disabled'); load(); }
      catch(e) { Toast.error(e.message); }
    },
  };
})();
