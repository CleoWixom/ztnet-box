// Physical Network Routing
// https://docs.zerotier.com/route-between-phys-and-virt/
const PhysnetPage = (() => {
  let _data = {};

  async function load() {
    const [plat, deps, status, ifaces] = await Promise.allSettled([
      api.get('/physnet/platform'),
      api.get('/physnet/deps'),
      api.get('/physnet/status'),
      api.get('/exitnode/interfaces'),
    ]);
    _data = {
      platform: plat.value,
      deps:     deps.value,
      status:   status.value,
      ifaces:   ifaces.value || [],
    };
    render();
  }

  function render() {
    const { platform, deps, status, ifaces } = _data;
    const el = document.getElementById('content');

    if (!platform?.supported) {
      el.innerHTML = `<div class="page"><div class="page-header"><h1 class="page-title">Physical Network Routing</h1></div>
        <div class="banner banner-warn">⚠️ Not supported: ${platform?.reason || platform?.os}</div></div>`;
      return;
    }

    const depsOk = deps && deps.iptables && deps.is_root;
    const physIfaces = (ifaces || []).filter(i => !i.is_zerotier && i.name !== 'lo');
    const ztIfaces   = (ifaces || []).filter(i => i.is_zerotier);

    el.innerHTML = `<div class="page">
      <div class="page-header">
        <div>
          <h1 class="page-title">Physical Network Routing</h1>
          <div class="text-dim text-sm">Route ZeroTier traffic to/from your physical LAN without touching your router</div>
        </div>
        <div style="display:flex;gap:var(--gap-sm);align-items:center">
          ${status?.enabled ? `<span class="badge badge-success">Active</span>` : `<span class="badge badge-muted">Inactive</span>`}
          <button class="btn btn-ghost btn-sm" onclick="this.closest('.page').querySelector('.help-box').classList.toggle('hidden')" title="Show/hide help">? Help</button>
        </div>
      </div>

      <div class="help-box hidden card mb">
        <div class="card-title mb-sm">What is Physical Routing?</div>
        <p class="text-sm text-dim mb-sm">Physical Routing allows remote ZeroTier peers to access devices on your <strong>physical LAN</strong> without installing ZeroTier on those devices. It uses NAT/Masquerade — unlike L2 Bridge, it works at Layer 3 and does not support broadcast or multicast.</p>
        <div class="card-title mb-sm mt">Requirements</div>
        <ul class="text-sm text-dim mb-sm" style="padding-left:1.2rem;line-height:1.8">
          <li>Linux with <code>iptables</code></li>
          <li>Running as root (sudo)</li>
          <li>A ZeroTier interface (<code>zt…</code>) and a physical/WAN interface</li>
        </ul>
        <div class="card-title mb-sm mt">How to set up</div>
        <ol class="text-sm text-dim" style="padding-left:1.2rem;line-height:1.8">
          <li>Select the ZeroTier interface (<code>zt…</code>) and WAN interface</li>
          <li>Enter the physical LAN subnet (e.g. <code>192.168.1.0/24</code>) and this node's ZeroTier IP</li>
          <li>Click <strong>Enable Physical Routing</strong></li>
          <li>In your controller: add a <strong>managed route</strong> for the physical subnet via this node's ZeroTier IP (use /23 — one size wider than the physical /24)</li>
        </ol>
        <p class="text-sm text-dim mt-sm">📖 <a href="https://docs.zerotier.com/route-between-phys-and-virt/" target="_blank">ZeroTier docs: Route between physical and virtual networks ↗</a></p>
      </div>

      <div class="section"><div class="section-title">Dependencies</div>
        <div class="card">
          <div class="toggle-wrap">
            <div><div class="toggle-label">iptables ${deps?.iptables ? '✅' : '❌'}</div></div>
          </div>
          <div class="toggle-wrap">
            <div><div class="toggle-label">Root access ${deps?.is_root ? '✅' : '❌'}</div>
              ${!deps?.is_root ? '<div class="toggle-hint text-sm text-dim">Restart ztnet-box with sudo</div>' : ''}</div>
          </div>
        </div>
      </div>

      ${status?.enabled ? `
      <div class="section"><div class="section-title">Status</div>
        <div class="card">
          <div class="detail-kv">
            <span class="k">ZT Interface</span><span class="v mono">${status.config?.zt_iface || '—'}</span>
            <span class="k">Physical Interface</span><span class="v mono">${status.config?.phy_iface || '—'}</span>
            <span class="k">Physical Subnet</span><span class="v mono">${status.config?.phy_subnet || '—'}</span>
            <span class="k">ZT Gateway IP</span><span class="v mono">${status.config?.zt_addr || '—'}</span>
            <span class="k">Since</span><span class="v">${status.applied_at ? new Date(status.applied_at).toLocaleString() : '—'}</span>
          </div>
          <div class="mt"><button class="btn btn-danger" onclick="PhysnetPage._disable()">Disable</button></div>
        </div>
      </div>` : `
      <div class="section"><div class="section-title">Configuration</div>
        <div class="card">
          <div class="field"><label class="field-label">ZeroTier Interface</label>
            <select class="select" id="pn-zt">
              ${ztIfaces.map(i => `<option value="${i.name}">${i.name}${i.addresses?.[0] ? ' — '+i.addresses[0] : ''}</option>`).join('')
                || '<option value="">No ZT interfaces found — join a network first</option>'}
            </select></div>
          <div class="field"><label class="field-label">Physical (WAN) Interface</label>
            <select class="select" id="pn-phy">
              ${physIfaces.map(i => `<option value="${i.name}">${i.name}${i.addresses?.[0] ? ' — '+i.addresses[0] : ''}</option>`).join('')
                || '<option value="">No physical interfaces found</option>'}
            </select></div>
          <div class="field"><label class="field-label">Physical LAN Subnet (CIDR)</label>
            <input class="input" id="pn-subnet" placeholder="e.g. 192.168.1.0/24">
            <div class="field-hint">The managed route in ZeroTier Central should use /23 (one size wider)</div></div>
          <div class="field"><label class="field-label">This node's ZeroTier IP</label>
            <input class="input" id="pn-ztaddr" placeholder="e.g. 172.27.0.1">
            <div class="field-hint">The gateway address — other ZT nodes will route physical traffic through this IP</div></div>
          <div class="field"><label class="field-label">ZeroTier Network ID</label>
            <input class="input" id="pn-netid" placeholder="16-digit hex">
          </div>
          <div id="pn-hint" class="banner banner-info" style="display:none"></div>
          <button class="btn btn-primary mt" onclick="PhysnetPage._enable()" ${!depsOk ? 'disabled' : ''}>
            Enable Physical Routing
          </button>
          ${!depsOk ? '<div class="text-sm text-dim mt-sm">Fix dependencies above first</div>' : ''}
        </div>
      </div>

      <div class="section"><div class="section-title">ZeroTier Central — Required Configuration</div>
        <div class="card">
          <div class="text-sm text-dim mb">After enabling, add this managed route in ZeroTier Central:</div>
          <div class="detail-kv">
            <span class="k">Destination</span><span class="v mono">your-physical-subnet/23 (one size wider)</span>
            <span class="k">Via</span><span class="v mono">this node's ZT IP</span>
          </div>
          <div class="text-sm text-dim mt-sm">Example: Physical LAN = 192.168.1.0/24 → Route 192.168.1.0/23 via 172.27.0.1</div>
        </div>
      </div>`}
    </div>`;

    // Update hint on subnet change
    document.getElementById('pn-subnet')?.addEventListener('input', function() {
      const hint = document.getElementById('pn-hint');
      if (!hint) return;
      const val = this.value.trim();
      if (val.includes('/')) {
        const parts = val.split('/');
        const prefix = parseInt(parts[1]);
        if (!isNaN(prefix) && prefix > 0) {
          hint.style.display = 'flex';
          hint.textContent = `💡 Add managed route in ZeroTier Central: ${parts[0]}/${prefix-1} via <your ZT IP>`;
        }
      } else {
        hint.style.display = 'none';
      }
    });
  }

  return {
    init() { load(); },
    async _enable() {
      const zt     = document.getElementById('pn-zt')?.value;
      const phy    = document.getElementById('pn-phy')?.value;
      const subnet = document.getElementById('pn-subnet')?.value?.trim();
      const addr   = document.getElementById('pn-ztaddr')?.value?.trim();
      const netid  = document.getElementById('pn-netid')?.value?.trim();
      if (!zt || !phy) return Toast.error('Select ZT and physical interfaces');
      if (!subnet)     return Toast.error('Enter physical subnet (CIDR)');
      if (!addr)       return Toast.error('Enter this node\'s ZeroTier IP');
      if (!netid)      return Toast.error('Enter network ID');
      try {
        const res = await api.post('/physnet/enable', {
          zt_iface: zt, phy_iface: phy,
          phy_subnet: subnet, zt_addr: addr, network_id: netid,
        });
        if (res.warnings?.length) res.warnings.forEach(w => Toast.info(w));
        Toast.success('Physical routing enabled');
        load();
      } catch(e) { Toast.error(e.message); }
    },
    async _disable() {
      if (!await Modal.confirm('Disable Physical Network Routing? iptables rules will be removed.', {danger:true})) return;
      try { await api.post('/physnet/disable'); Toast.success('Disabled'); load(); }
      catch(e) { Toast.error(e.message); }
    },
  };
})();
