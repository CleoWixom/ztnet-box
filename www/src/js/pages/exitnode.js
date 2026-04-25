const ExitnodePage = (() => {
  let _data = {};

  async function load() {
    const [platform, deps, ifaces, status, ndp] = await Promise.allSettled([
      api.get('/exitnode/platform'),
      api.get('/exitnode/deps'),
      api.get('/exitnode/interfaces'),
      api.get('/exitnode/status'),
      api.get('/exitnode/ndp/status'),
    ]);
    _data = {
      platform: platform.value,
      deps:     deps.value,
      ifaces:   ifaces.value || [],
      status:   status.value,
      ndp:      ndp.value || null,
    };
    render();
  }

  function render() {
    const { platform, deps, ifaces, status, ndp } = _data;
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

    const nets = State.get('networks') || [];
    const netOpts = nets.filter(n=>n.status==='OK'||n._src).map(n=>`<option value="${n.id}">${n.id} ${n.name?('('+n.name+')'):''}  </option>`).join('');

    // ZeroTier interfaces (zt*) for the zt_interface field — separate from network ID
    const ztIfaces = (ifaces||[]).filter(i=>i.is_zerotier);
    const ztIfaceOpts = ztIfaces.map(i=>`<option value="${i.name}">${i.name}</option>`).join('');

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
        <div class="text-dim text-sm">Scope FORWARD rules to this prefix. Leave empty for wildcard.</div>
      </div>`;

    const statusBlock = status ? `
      <div class="card mt">
        <div class="card-header"><div class="card-title">Status</div>
          <span class="badge ${status.enabled?'badge-success':'badge-muted'}">${status.enabled?'Enabled':'Disabled'}</span>
        </div>
        ${status.enabled?`
          <div class="detail-kv">
            <span class="k">ZT Interface</span><span class="v mono">${status.zt_interface||'—'}</span>
            <span class="k">WAN Interface</span><span class="v mono">${status.wan_interface||'—'}</span>
            <span class="k">Backend</span><span class="v">${status.backend||'—'}</span>
            <span class="k">IPv6</span><span class="v">${status.enable_ipv6?('✓ '+(status.ipv6_prefix||'(all prefixes)')):'Disabled'}</span>
            <span class="k">Since</span><span class="v">${status.applied_at?new Date(status.applied_at).toLocaleString():'—'}</span>
          </div>`:'<div class="text-dim text-sm">Exit Node is not active.</div>'}
      </div>` : '';

    // NDP Proxy section
    const ndpSt = ndp || {};
    const ndpWanOpts = wanIfaces.map(i=>`<option value="${i.name}">${i.name}</option>`).join('');
    const ndpBadge = ndpSt.running
      ? '<span class="badge badge-success">Running</span>'
      : '<span class="badge badge-muted">Stopped</span>';
    const ndpSection = `
      <div class="section"><div class="section-title">NDP Proxy (ndppd) <small class="text-dim">— native IPv6 without NAT</small></div>
        <div class="card">
          <div class="text-dim text-sm mb-sm">
            Answers IPv6 Neighbor Discovery requests on WAN for ZeroTier client addresses,
            enabling real IPv6 routing without MASQUERADE.
          </div>
          <div class="detail-kv mb-sm">
            <span class="k">ndppd</span>
            <span class="v">${ndpSt.available?`✓ ${ndpSt.binary_path||''}`:'✗ Not installed'}</span>
            <span class="k">Status</span><span class="v">${ndpBadge}</span>
            <span class="k">Config</span>
            <span class="v">${ndpSt.config_exists?'/etc/ndppd.conf ✓':'Not configured'}</span>
          </div>
          ${!ndpSt.available?`
            <button class="btn btn-secondary btn-sm" onclick="ExitnodePage._ndpInstall()">Install ndppd</button>`:''}
          ${ndpSt.available&&!ndpSt.running?`
            <div class="field mt-sm"><label class="field-label">WAN Interface</label>
              <select class="select" id="ndp-wan" style="max-width:200px">
                ${ndpWanOpts||'<option value="">No interfaces</option>'}
              </select></div>
            <div class="field"><label class="field-label">IPv6 Prefix <span class="text-dim">(CIDR)</span></label>
              <input class="input" id="ndp-prefix" placeholder="2001:db8::/64"></div>
            <button class="btn btn-primary mt-sm" onclick="ExitnodePage._ndpEnable()">Enable NDP Proxy</button>`:''}
          ${ndpSt.running?`
            <button class="btn btn-danger btn-sm mt-sm" onclick="ExitnodePage._ndpDisable()">Disable NDP Proxy</button>`:''}
        </div>
      </div>`;

    el.innerHTML = `<div class="page">
      <div class="page-header">
        <h1 class="page-title">Exit Node</h1>
        <button class="btn btn-ghost btn-sm" onclick="this.closest('.page').querySelector('.help-box').classList.toggle('hidden')" title="Show/hide help">? Help</button>
      </div>
      <div class="help-box hidden card mb">
        <div class="card-title mb-sm">What is an Exit Node?</div>
        <p class="text-sm text-dim mb-sm">An Exit Node routes all internet traffic from other ZeroTier members through this machine — similar to a self-hosted VPN server. Members that enable <em>Default Route</em> on the shared network will send all traffic here.</p>
        <div class="card-title mb-sm mt">Requirements</div>
        <ul class="text-sm text-dim mb-sm" style="padding-left:1.2rem;line-height:1.8">
          <li>Linux only (nftables or iptables must be installed)</li>
          <li>Running as root (sudo) — required for firewall rule management</li>
          <li>IP forwarding enabled (set automatically)</li>
          <li>A WAN interface with internet access (eth0, ens3, …)</li>
        </ul>
        <div class="card-title mb-sm mt">How to set up</div>
        <ol class="text-sm text-dim" style="padding-left:1.2rem;line-height:1.8">
          <li>Install missing dependencies (iptables / nftables) if shown below</li>
          <li>Select the ZeroTier interface (<code>zt…</code>) and your WAN interface</li>
          <li>Click <strong>Enable Exit Node</strong></li>
          <li>In ZeroTier Central or your local controller: set <strong>Allow Default Route</strong> for the network</li>
          <li>On member devices: enable the <em>Default Route</em> option for that network</li>
        </ol>
      </div>
      <div class="section"><div class="section-title">Dependencies</div>
        <div class="card"><div class="step-list">${depsList}</div></div>
      </div>
      <div class="section"><div class="section-title">Configuration</div>
        <div class="card">
          <div class="field"><label class="field-label">ZeroTier Interface</label>
            <select class="select" id="en-zt-iface">${ztIfaceOpts||'<option value="">No ZT interfaces detected</option>'}</select>
            <div class="text-dim text-sm">Interface name (zt…), not the network ID</div></div>
          <div class="field"><label class="field-label">ZeroTier Network ID <span class="text-dim">(optional — for allowDefault check)</span></label>
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
      ${ndpSection}
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
      try { await api.post('/exitnode/deps'); Toast.success('Dependencies installed'); load(); }
      catch(e) { Toast.error(e.message); }
    },
    async _enable() {
      const ztIface = document.getElementById('en-zt-iface')?.value;
      const netId   = document.getElementById('en-net')?.value || null;
      const wan     = document.getElementById('en-wan')?.value;
      const ipv6    = document.getElementById('en-ipv6')?.checked || false;
      const ipv6_prefix = document.getElementById('en-ipv6-prefix')?.value.trim() || null;
      if (!ztIface) return Toast.error('Select a ZeroTier interface (zt…)');
      if (!wan)     return Toast.error('Select a WAN interface');
      const ipv6note = ipv6 ? '<br><small>IPv6 ip6tables rules will also be applied.</small>' : '';
      if (!await Modal.confirm(`Enable Exit Node?<br><small>Route traffic on <b>${ztIface}</b> through <b>${wan}</b>.</small>${ipv6note}`)) return;
      try {
        const res = await api.post('/exitnode/enable', {
          zt_interface: ztIface, wan_interface: wan, network_id: netId,
          enable_ipv6: ipv6, ipv6_prefix,
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
    async _ndpInstall() {
      try { await api.post('/exitnode/ndp/install'); Toast.success('ndppd installed'); load(); }
      catch(e) { Toast.error(e.message); }
    },
    async _ndpEnable() {
      const wan = document.getElementById('ndp-wan')?.value;
      const prefix = document.getElementById('ndp-prefix')?.value.trim();
      if (!wan) return Toast.error('Select a WAN interface');
      if (!prefix) return Toast.error('Enter an IPv6 prefix (CIDR)');
      try {
        await api.post('/exitnode/ndp/enable', { wan_iface: wan, ipv6_prefix: prefix });
        Toast.success('NDP Proxy enabled');
        load();
      } catch(e) { Toast.error(e.message); }
    },
    async _ndpDisable() {
      if (!await Modal.confirm('Disable NDP Proxy?', {danger:true})) return;
      try {
        await api.post('/exitnode/ndp/disable', { remove_config: false });
        Toast.success('NDP Proxy disabled');
        load();
      } catch(e) { Toast.error(e.message); }
    },
  };
})();
