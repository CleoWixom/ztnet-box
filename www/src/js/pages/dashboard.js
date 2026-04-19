const DashboardPage = (() => {
  let _intervals = [];

  function fmtBytes(b) {
    if (b < 1024) return b + ' B';
    if (b < 1048576) return (b/1024).toFixed(1) + ' KB';
    if (b < 1073741824) return (b/1048576).toFixed(1) + ' MB';
    return (b/1073741824).toFixed(2) + ' GB';
  }

  function latencyClass(ms) {
    if (ms < 50) return 'latency-good';
    if (ms < 150) return 'latency-medium';
    return 'latency-bad';
  }

  function renderStatus(node) {
    if (!node) return '<div class="loading-row"><div class="spinner"></div> Loading node status...</div>';
    return `<div class="node-status-bar">
      <div>
        <div class="node-id">${node.address}</div>
        <div class="text-sm text-dim">ZeroTier Node</div>
      </div>
      <div class="node-status-fields">
        <div class="node-field"><span class="node-field-label">Status</span>
          <span>${node.online ? '<span class="badge badge-success"><span class="dot"></span> Online</span>' : '<span class="badge badge-danger"><span class="dot"></span> Offline</span>'}</span></div>
        <div class="node-field"><span class="node-field-label">Version</span><span class="node-field-val">${node.version || '—'}</span></div>
        <div class="node-field"><span class="node-field-label">World ID</span><span class="node-field-val mono">${node.worldId || node.world_id || '—'}</span></div>
        <div class="node-field"><span class="node-field-label">TCP Fallback</span><span class="node-field-val">${node.tcpFallbackActive ? 'Active' : 'Direct'}</span></div>
      </div>
    </div>`;
  }

  function renderMetrics(m, status) {
    if (!status?.enabled) return `<div class="banner banner-info">ℹ️ Metrics collection is disabled. Enable it in Settings → Global.</div>`;
    if (!m) return '<div class="loading-row"><div class="spinner"></div> Waiting for metrics...</div>';
    const age = status?.last_updated ? `Updated ${new Date(status.last_updated).toLocaleTimeString()}` : '';
    return `<div class="cards-grid mb">
      <div class="metric-card">
        <div class="metric-card-label">RX Traffic</div>
        <div class="metric-card-value">${fmtBytes(m.packets?.rx_bytes||0)}</div>
        <div class="metric-card-sub">${fmtBytes(m.packets?.rx_packets||0)} packets</div>
      </div>
      <div class="metric-card">
        <div class="metric-card-label">TX Traffic</div>
        <div class="metric-card-value">${fmtBytes(m.packets?.tx_bytes||0)}</div>
        <div class="metric-card-sub">${fmtBytes(m.packets?.tx_packets||0)} packets</div>
      </div>
      <div class="metric-card">
        <div class="metric-card-label">Avg Latency</div>
        <div class="metric-card-value ${latencyClass(m.latency?.avg_ms||0)}">${(m.latency?.avg_ms||0).toFixed(1)} ms</div>
        <div class="metric-card-sub">min ${(m.latency?.min_ms||0).toFixed(0)} / max ${(m.latency?.max_ms||0).toFixed(0)} ms</div>
      </div>
      <div class="metric-card">
        <div class="metric-card-label">Packet Errors</div>
        <div class="metric-card-value">${m.errors?.total||0}</div>
        <div class="metric-card-sub">${age}</div>
      </div>
    </div>`;
  }

  function renderPeers(peers) {
    if (!peers?.length) return `<div class="empty-state"><div class="empty-state-icon">🤝</div><h3>No peers</h3><p>Connect to a ZeroTier network to see peers.</p></div>`;
    const rows = peers.map(p => {
      const roleClass = p.role === 'PLANET' ? 'badge-info' : p.role === 'MOON' ? 'badge-warn' : 'badge-muted';
      const latMs = p.latency >= 0 ? p.latency : '—';
      const latClass = p.latency >= 0 ? latencyClass(p.latency) : '';
      return `<tr>
        <td><span class="mono">${p.address}</span></td>
        <td><span class="badge ${roleClass}">${p.role}</span></td>
        <td class="${latClass}">${latMs}${typeof latMs === 'number' ? ' ms' : ''}</td>
        <td>${p.paths?.filter(x=>x.active).length || 0}</td>
        <td>${p.version || '—'}</td>
      </tr>`;
    }).join('');
    return `<div class="table-wrap"><table>
      <thead><tr><th>Node ID</th><th>Role</th><th>Latency</th><th>Paths</th><th>Version</th></tr></thead>
      <tbody>${rows}</tbody></table></div>`;
  }

  function render() {
    const el = document.getElementById('content');
    el.innerHTML = `<div class="page">
      <div class="page-header"><h1 class="page-title">Dashboard</h1></div>
      <div id="dash-status"></div>
      <div class="section"><div class="section-title">Metrics</div><div id="dash-metrics"></div></div>
      <div class="section">
        <div class="section-title" style="display:flex;justify-content:space-between">
          Peers <span class="text-sm text-dim" id="peer-count"></span>
        </div>
        <div id="dash-peers"></div>
      </div>
    </div>`;

    refresh();
    _intervals.push(setInterval(refresh, 10000));
  }

  // Lifted to IIFE scope so _installZt() can call it after install completes
  async function refresh() {
      // Check ZeroTier installation first — show setup banner if not found
      try {
        const zt = await api.get('/system/zt-status');
        if (!zt.cli_available) {
          document.getElementById('dash-status').innerHTML = `
            <div class="banner banner-warn">
              ⚠️ ZeroTier is not installed or not running on this host.
              <button class="btn btn-sm btn-primary ml-sm" onclick="DashboardPage._installZt(this)">Install ZeroTier</button>
            </div>`;
          return;
        }
      } catch(e) {}

      // Fetch node status, metrics and peers in parallel
      const [nodeRes, metricsRes, msRes, peersRes] = await Promise.allSettled([
        api.get('/local/status'),
        api.get('/metrics'),
        api.get('/metrics/status'),
        api.get('/local/peers'),
      ]);

      if (nodeRes.status === 'fulfilled') {
        State.set('nodeStatus', nodeRes.value);
        document.getElementById('dash-status').innerHTML = renderStatus(nodeRes.value);
      } else {
        document.getElementById('dash-status').innerHTML =
          `<div class="banner banner-danger">❌ Cannot reach ZeroTier: ${nodeRes.reason?.message||'unknown'}</div>`;
      }

      if (metricsRes.status === 'fulfilled' || msRes.status === 'fulfilled') {
        const m  = metricsRes.value  || null;
        const ms = msRes.value || null;
        State.set('metrics', m); State.set('metricsStatus', ms);
        document.getElementById('dash-metrics').innerHTML = renderMetrics(m, ms);
      }

      if (peersRes.status === 'fulfilled') {
        const peers = peersRes.value;
        State.set('peers', peers);
        document.getElementById('dash-peers').innerHTML = renderPeers(peers);
        const el = document.getElementById('peer-count');
        if (el) el.textContent = peers.length + ' peer' + (peers.length !== 1 ? 's' : '');
      }
    }

  return {
    init() { render(); },
    destroy() { _intervals.forEach(clearInterval); _intervals = []; },
    async _installZt(btn) {
      if (btn) { btn.disabled = true; btn.textContent = 'Installing…'; }
      try {
        const res = await api.post('/system/zt-install', {});
        if (res.status === 'installed' || res.status === 'already_installed') {
          Toast.success('ZeroTier installed — refreshing…');
          setTimeout(() => refresh(), 1500);
        } else {
          Toast.error(res.reason || 'Install failed');
          if (btn) { btn.disabled = false; btn.textContent = 'Install ZeroTier'; }
        }
      } catch(e) {
        Toast.error(e.message);
        if (btn) { btn.disabled = false; btn.textContent = 'Install ZeroTier'; }
      }
    },
  };
})();
