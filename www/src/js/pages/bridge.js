const BridgePage = (() => {
  let _data = {};

  async function load() {
    const [platform, deps, ifaces, status] = await Promise.allSettled([
      api.get('/bridge/platform'),
      api.get('/bridge/deps'),
      api.get('/exitnode/interfaces'),
      api.get('/bridge/status'),
    ]);
    _data = {
      platform: platform.value,
      deps:     deps.value,
      ifaces:   ifaces.value || [],
      status:   status.value,
    };
    render();
  }

  function render() {
    const { platform, deps, ifaces, status } = _data;
    const el = document.getElementById('content');

    if (!platform?.supported) {
      el.innerHTML = `<div class="page"><div class="page-header"><h1 class="page-title">L2 Bridge</h1></div>
        <div class="banner banner-warn">⚠️ Not supported on this platform: ${platform?.reason || platform?.os || 'unknown'}</div></div>`;
      return;
    }

    const depOk = (deps?.systemd_networkd && deps?.iproute2 && !deps?.dhcpcd_conflict && !deps?.ifupdown_conflict);
    const depsList = deps ? `
      <div class="step-item${deps.iproute2 ? ' done' : ''}">
        <div class="step-num">${deps.iproute2 ? '✓' : '1'}</div>
        <div class="step-content">
          <div class="step-title">iproute2 (ip command)</div>
          <div class="text-sm">${deps.iproute2 ? 'Available ✓' : 'Not found'}</div>
        </div>
      </div>
      <div class="step-item${deps.systemd_networkd ? ' done' : ''}">
        <div class="step-num">${deps.systemd_networkd ? '✓' : '2'}</div>
        <div class="step-content">
          <div class="step-title">systemd-networkd</div>
          <div class="text-sm">${deps.systemd_networkd ? 'Available ✓' : 'Not found'}</div>
        </div>
      </div>
      <div class="step-item${!deps.dhcpcd_conflict && !deps.ifupdown_conflict ? ' done' : ''}">
        <div class="step-num">${!deps.dhcpcd_conflict && !deps.ifupdown_conflict ? '✓' : '3'}</div>
        <div class="step-content">
          <div class="step-title">No conflicting services</div>
          <div class="text-sm">${deps.dhcpcd_conflict ? '⚠ dhcpcd installed' : ''} ${deps.ifupdown_conflict ? '⚠ ifupdown installed' : ''} ${!deps.dhcpcd_conflict && !deps.ifupdown_conflict ? 'Clean ✓' : ''}</div>
          ${(deps.dhcpcd_conflict || deps.ifupdown_conflict) ? `<button class="btn btn-warn btn-sm mt-sm" onclick="BridgePage._installDeps()">Remove conflicts</button>` : ''}
        </div>
      </div>
      <div class="step-item${deps.is_root ? ' done' : ''}">
        <div class="step-num">${deps.is_root ? '✓' : '4'}</div>
        <div class="step-content">
          <div class="step-title">Root Access</div>
          <div class="text-sm">${deps.is_root ? 'Running as root ✓' : 'Restart with sudo'}</div>
        </div>
      </div>` : '<div class="loading-row"><div class="spinner"></div> Checking dependencies...</div>';

    const ztIfaces = (ifaces || []).filter(i => i.is_zerotier);
    const phyIfaces = (ifaces || []).filter(i => !i.is_zerotier && i.name !== 'lo');
    const ztOpts = ztIfaces.map(i => `<option value="${i.name}">${i.name} ${i.addresses?.[0] ? '— ' + i.addresses[0] : ''}</option>`).join('');
    const phyOpts = phyIfaces.map(i => `<option value="${i.name}">${i.name} ${i.addresses?.[0] ? '— ' + i.addresses[0] : ''}</option>`).join('');
    const nets = State.get('networks') || [];
    const netOpts = nets.map(n => `<option value="${n.id}">${n.id} ${n.name ? '(' + n.name + ')' : ''}</option>`).join('');

    const statusBlock = status ? `
      <div class="card mt">
        <div class="card-header"><div class="card-title">Status</div>
          <span class="badge ${status.enabled ? 'badge-success' : 'badge-muted'}">${status.enabled ? 'Enabled' : 'Disabled'}</span>
        </div>
        ${status.enabled && status.config ? `
          <div class="detail-kv">
            <span class="k">ZT Interface</span><span class="v mono">${status.config.zt_iface}</span>
            <span class="k">Physical Interface</span><span class="v mono">${status.config.phy_iface}</span>
            <span class="k">Bridge Interface</span><span class="v mono">${status.config.bridge_iface}</span>
            <span class="k">Bridge Address</span><span class="v mono">${status.config.bridge_addr || 'DHCP'}</span>
            <span class="k">Gateway</span><span class="v mono">${status.config.gateway || '—'}</span>
            <span class="k">ZT Network</span><span class="v mono">${status.config.network_id}</span>
            <span class="k">Since</span><span class="v">${status.applied_at ? new Date(status.applied_at).toLocaleString() : '—'}</span>
          </div>` : '<div class="text-dim text-sm">Bridge is not active.</div>'}
      </div>` : '';

    const infoBox = `<div class="banner banner-info mt-sm">
      ℹ️ L2 Bridge connects physical LAN hosts to the ZeroTier network at layer 2.
      After enabling, set <b>bridging=true</b> for this member in ZeroTier Central.
      Requires systemd-networkd; incompatible with dhcpcd/ifupdown.
    </div>`;

    el.innerHTML = `<div class="page">
      <div class="page-header">
        <h1 class="page-title">L2 Bridge</h1>
        <button class="btn btn-ghost btn-sm" onclick="this.closest('.page').querySelector('.help-box').classList.toggle('hidden')" title="Show/hide help">? Help</button>
      </div>
      <div class="help-box hidden card mb">
        <div class="card-title mb-sm">What is L2 Bridge?</div>
        <p class="text-sm text-dim mb-sm">L2 Bridge connects a ZeroTier virtual network to a physical LAN at Layer 2. Devices on your local network become directly reachable from ZeroTier without needing ZeroTier installed — as if the ZT network and your LAN were the same switch.</p>
        <div class="card-title mb-sm mt">Requirements</div>
        <ul class="text-sm text-dim mb-sm" style="padding-left:1.2rem;line-height:1.8">
          <li>Linux only (systemd-networkd + iproute2)</li>
          <li>No conflicting tools: <code>dhcpcd</code> and <code>ifupdown</code> must not manage the bridge interface</li>
          <li>Running as root — bridge interface creation requires elevated privileges</li>
          <li>A separate physical NIC or VLAN to bridge onto</li>
        </ul>
        <div class="card-title mb-sm mt">How to set up</div>
        <ol class="text-sm text-dim" style="padding-left:1.2rem;line-height:1.8">
          <li>Resolve any dependency conflicts shown below</li>
          <li>Select the ZeroTier interface (<code>zt…</code>) and the physical interface to bridge</li>
          <li>Name the bridge interface (default: <code>br0</code>)</li>
          <li>Optionally assign a static IP/gateway to the bridge</li>
          <li>Click <strong>Enable Bridge</strong></li>
          <li>In your controller, enable <strong>Allow Bridging</strong> for this member</li>
        </ol>
      </div>
      <div class="section"><div class="section-title">Dependencies</div>
        <div class="card"><div class="step-list">${depsList}</div></div>
      </div>
      <div class="section"><div class="section-title">Configuration</div>
        <div class="card">
          <div class="field"><label class="field-label">ZeroTier Interface</label>
            <select class="select" id="br-zt">${ztOpts || '<option value="">No ZT interfaces</option>'}</select></div>
          <div class="field"><label class="field-label">Physical Interface</label>
            <select class="select" id="br-phy">${phyOpts || '<option value="">No physical interfaces</option>'}</select></div>
          <div class="field"><label class="field-label">Bridge Interface</label>
            <input class="input" id="br-iface" value="${status?.config?.bridge_iface || 'br0'}"></div>
          <div class="field"><label class="field-label">ZeroTier Network</label>
            <select class="select" id="br-net">${netOpts || '<option value="">No networks</option>'}</select></div>
          <div class="field"><label class="field-label">Bridge Address <span class="text-dim">(optional, CIDR)</span></label>
            <input class="input" id="br-addr" placeholder="e.g. 192.168.1.10/24" value="${status?.config?.bridge_addr || ''}"></div>
          <div class="field"><label class="field-label">Gateway <span class="text-dim">(optional)</span></label>
            <input class="input" id="br-gw" placeholder="e.g. 192.168.1.1" value="${status?.config?.gateway || ''}"></div>
          ${status?.enabled
            ? `<button class="btn btn-danger mt" onclick="BridgePage._disable()">Disable Bridge</button>`
            : `<button class="btn btn-primary mt" onclick="BridgePage._enable()" ${depOk ? '' : 'disabled title="Fix dependencies first"'}>Enable Bridge</button>`}
        </div>
      </div>
      ${statusBlock}
    </div>`;
  }

  return {
    async init() {
      document.getElementById('content').innerHTML = '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading...</div></div>';
      try { const nets = await api.get('/local/networks'); State.set('networks', nets); } catch (e) {}
      await load();
    },
    async _installDeps() {
      try { await api.post('/bridge/deps/install'); Toast.success('Conflicts removed'); load(); }
      catch (e) { Toast.error(e.message); }
    },
    async _enable() {
      const zt    = document.getElementById('br-zt')?.value;
      const phy   = document.getElementById('br-phy')?.value;
      const br    = document.getElementById('br-iface')?.value || 'br0';
      const net   = document.getElementById('br-net')?.value;
      const addr  = document.getElementById('br-addr')?.value.trim() || null;
      const gw    = document.getElementById('br-gw')?.value.trim() || null;
      if (!zt)  return Toast.error('Select a ZeroTier interface');
      if (!phy) return Toast.error('Select a physical interface');
      if (!net) return Toast.error('Select a ZeroTier network');
      if (!await Modal.confirm(
        `Enable L2 Bridge?<br><small>Bridge <b>${br}</b> will join <b>${zt}</b> and <b>${phy}</b>.<br>
         Ensure no other DHCP clients are running.</small>`)) return;
      try {
        const res = await api.post('/bridge/enable', {
          zt_iface: zt, phy_iface: phy, bridge_iface: br,
          network_id: net, bridge_addr: addr, gateway: gw,
        });
        Toast.success('Bridge enabled');
        if (res.next_step) Toast.info(res.next_step);
        load();
      } catch (e) { Toast.error(e.message); }
    },
    async _disable() {
      if (!await Modal.confirm('Disable L2 Bridge?', { danger: true })) return;
      try { await api.post('/bridge/disable'); Toast.success('Bridge disabled'); load(); }
      catch (e) { Toast.error(e.message); }
    },
  };
})();
