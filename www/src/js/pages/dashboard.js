const DashboardPage = (() => {
  let _intervals = [];

  // ── Helpers ──────────────────────────────────────────────────────────────────

  function fmtBytes(b) {
    if (b < 1024) return b + ' B';
    if (b < 1048576) return (b/1024).toFixed(1) + ' KB';
    if (b < 1073741824) return (b/1048576).toFixed(1) + ' MB';
    return (b/1073741824).toFixed(2) + ' GB';
  }

  // ── Renderers ─────────────────────────────────────────────────────────────────

  function renderStatus(node) {
    if (!node) return '<div class="loading-row"><div class="spinner"></div> Loading node status...</div>';
    const statusBadge = node.online
      ? '<span class="badge badge-success"><span class="dot"></span> Online</span>'
      : '<span class="badge badge-danger"><span class="dot"></span> Offline</span>';
    return `<div class="node-status-bar">
      <div>
        <div class="node-id">${Utils.esc(node.address)}</div>
        <div class="text-sm text-dim">ZeroTier Node</div>
      </div>
      <div class="node-status-fields">
        <div class="node-field"><span class="node-field-label">Status</span><span>${statusBadge}</span></div>
        <div class="node-field"><span class="node-field-label">Version</span><span class="node-field-val">${Utils.esc(node.version || '—')}</span></div>
        <div class="node-field"><span class="node-field-label">World ID</span><span class="node-field-val mono">${Utils.esc(String(node.worldId || node.world_id || '—'))}</span></div>
        <div class="node-field"><span class="node-field-label">TCP Fallback</span><span class="node-field-val">${node.tcpFallbackActive ? 'Active' : 'Direct'}</span></div>
      </div>
      <div class="join-inline">
        <input class="input input-sm" id="dash-join-input" placeholder="Network ID (16 hex)" maxlength="16">
        <button class="btn btn-primary btn-sm" onclick="DashboardPage._join()">Join</button>
      </div>
    </div>`;
  }

  function renderMetrics(m, status) {
    if (!status?.enabled) return `<div class="banner banner-info">ℹ️ Metrics disabled — enable in Settings → Global.</div>`;
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
        <div class="metric-card-value ${Utils.latencyClass(m.latency?.avg_ms||0)}">${(m.latency?.avg_ms||0).toFixed(1)} ms</div>
        <div class="metric-card-sub">min ${(m.latency?.min_ms||0).toFixed(0)} / max ${(m.latency?.max_ms||0).toFixed(0)} ms</div>
      </div>
      <div class="metric-card">
        <div class="metric-card-label">Packet Errors</div>
        <div class="metric-card-value">${m.errors?.total||0}</div>
        <div class="metric-card-sub">${age}</div>
      </div>
    </div>`;
  }

  // Network card with member dots
  function renderNetworkCard(net, members) {
    const id    = Utils.esc(net.id || '');
    const name  = Utils.esc(net.name || net.config?.name || '');
    const ips   = (net.assignedAddresses || net.config?.assignedAddresses || []).join(', ') || '—';
    const status = net.status || 'OK';
    const badgeClass = status === 'OK' ? 'badge-success' : 'badge-warn';

    // Members online = lastOnline within last 5 min
    let memberHtml = '';
    if (members) {
      const now = Date.now();
      const threshold = 5 * 60 * 1000;
      const online  = members.filter(m => {
        const t = m.lastOnline ?? m.last_online ?? m.lastSeen ?? 0;
        return t && (now - t) < threshold;
      }).length;
      const total = members.length;

      // First 8 dots: green = online, grey = offline
      const dots = members.slice(0, 8).map(m => {
        const t = m.lastOnline ?? m.last_online ?? m.lastSeen ?? 0;
        const isOn = t && (now - t) < threshold;
        return `<span class="member-dot ${isOn ? 'online' : ''}" title="${Utils.esc(m.node_id||m.nodeId||'')}"></span>`;
      }).join('');
      const more = total > 8 ? `<span class="text-mute text-sm">+${total - 8}</span>` : '';

      memberHtml = `<div class="net-card-members">
        <span class="net-card-members-count">${online}<span class="text-mute">/${total}</span> online</span>
        <span class="member-dots">${dots}${more}</span>
      </div>`;
    }

    return `<div class="net-card" onclick="Router.navigate('/networks/${id}')" tabindex="0" role="button">
      <div class="net-card-head">
        <span class="mono net-card-id">${id}</span>
        <span class="badge ${badgeClass}">${Utils.esc(status)}</span>
      </div>
      ${name ? `<div class="net-card-name">${name}</div>` : ''}
      <div class="net-card-ip text-sm text-dim">${Utils.esc(ips)}</div>
      ${memberHtml}
      <div class="net-card-footer">
        <button class="btn btn-ghost btn-sm" onclick="event.stopPropagation();Router.navigate('/networks/${id}')">Details →</button>
        <button class="btn btn-danger btn-sm" onclick="event.stopPropagation();DashboardPage._leave('${id}')">Leave</button>
      </div>
    </div>`;
  }

  function renderNetworks(nets, membersMap) {
    if (!nets?.length) return `<div class="empty-state">
      <div class="empty-state-icon">🌐</div>
      <h3>Not connected to any network</h3>
      <p>Enter a 16-character Network ID above and click Join.</p>
    </div>`;
    return `<div class="net-cards-grid">${nets.map(n => renderNetworkCard(n, membersMap[n.id])).join('')}</div>`;
  }

  function renderPeers(peers) {
    if (!peers?.length) return `<div class="empty-state"><div class="empty-state-icon">🤝</div><h3>No peers</h3><p>Connect to a network to see peers.</p></div>`;
    const rows = peers.map(p => {
      const roleClass = p.role === 'PLANET' ? 'badge-info' : p.role === 'MOON' ? 'badge-warn' : 'badge-muted';
      const latMs  = p.latency >= 0 ? p.latency : '—';
      const latCls = p.latency >= 0 ? Utils.latencyClass(p.latency) : '';
      return `<tr>
        <td><span class="mono">${Utils.esc(p.address)}</span></td>
        <td><span class="badge ${roleClass}">${Utils.esc(p.role)}</span></td>
        <td class="${latCls}">${latMs}${typeof latMs === 'number' ? ' ms' : ''}</td>
        <td>${p.paths?.filter(x=>x.active).length || 0}</td>
        <td>${Utils.esc(p.version || '—')}</td>
        <td class="text-sm text-dim mono">${Utils.esc(p.physicalAddress || p.physical_address || '—')}</td>
      </tr>`;
    }).join('');
    return `<div class="table-wrap"><table>
      <thead><tr><th>Node ID</th><th>Role</th><th>Latency</th><th>Paths</th><th>Version</th><th>Physical IP</th></tr></thead>
      <tbody>${rows}</tbody></table></div>`;
  }

  // ── Page structure ─────────────────────────────────────────────────────────────

  function render() {
    document.getElementById('content').innerHTML = `<div class="page">
      <div class="page-header"><h1 class="page-title">Dashboard</h1></div>
      <div id="dash-status"></div>
      <div class="section"><div class="section-title">Metrics</div><div id="dash-metrics"></div></div>
      <div class="section">
        <div class="section-title" style="display:flex;justify-content:space-between">
          Networks <span class="text-sm text-dim" id="net-count"></span>
        </div>
        <div id="dash-networks"></div>
      </div>
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

  // ── Refresh ────────────────────────────────────────────────────────────────────

  async function refresh() {
    // ZeroTier install check
    try {
      const zt = await api.get('/system/zt-status');
      if (!zt.cli_available) {
        document.getElementById('dash-status').innerHTML = `
          <div class="banner banner-warn">
            ⚠️ ZeroTier is not installed or not running on this host.
            <button class="btn btn-sm btn-primary" style="margin-left:8px" onclick="DashboardPage._installZt(this)">Install ZeroTier</button>
          </div>`;
        return;
      }
    } catch {}

    // Parallel fetch: status + metrics + peers + networks
    const [nodeRes, metricsRes, msRes, peersRes, netsRes] = await Promise.allSettled([
      api.get('/local/status'),
      api.get('/metrics'),
      api.get('/metrics/status'),
      api.get('/local/peers'),
      api.get('/local/networks'),
    ]);

    // Node status
    if (nodeRes.status === 'fulfilled') {
      State.set('nodeStatus', nodeRes.value);
      document.getElementById('dash-status').innerHTML = renderStatus(nodeRes.value);
    } else {
      document.getElementById('dash-status').innerHTML =
        `<div class="banner banner-danger">❌ Cannot reach ZeroTier: ${Utils.esc(nodeRes.reason?.message || 'unknown')}</div>`;
    }

    // Metrics
    if (metricsRes.status === 'fulfilled' || msRes.status === 'fulfilled') {
      const m  = metricsRes.value || null;
      const ms = msRes.value || null;
      State.set('metrics', m); State.set('metricsStatus', ms);
      document.getElementById('dash-metrics').innerHTML = renderMetrics(m, ms);
    }

    // Peers
    if (peersRes.status === 'fulfilled') {
      const peers = peersRes.value || [];
      State.set('peers', peers);
      document.getElementById('dash-peers').innerHTML = renderPeers(peers);
      const el = document.getElementById('peer-count');
      if (el) el.textContent = peers.length + ' peer' + (peers.length !== 1 ? 's' : '');
    }

    // Networks + member counts
    if (netsRes.status === 'fulfilled') {
      const nets = netsRes.value || [];
      State.set('networks', nets);
      const nc = document.getElementById('net-count');
      if (nc) nc.textContent = nets.length + ' network' + (nets.length !== 1 ? 's' : '');

      // Fetch member counts in parallel (best-effort; requires local controller or Central token)
      const membersMap = {};
      await Promise.allSettled(nets.map(async n => {
        try {
          // Try local controller first
          const members = await api.get(`/local/controller/networks/${n.id}/members`);
          membersMap[n.id] = members;
        } catch {
          try {
            // Fallback to Central API
            const members = await api.get(`/central/networks/${n.id}/members`);
            membersMap[n.id] = members;
          } catch {}
        }
      }));

      const el = document.getElementById('dash-networks');
      if (el) el.innerHTML = renderNetworks(nets, membersMap);
    }
  }

  // ── Public API ─────────────────────────────────────────────────────────────────

  return {
    init() { render(); },
    destroy() { _intervals.forEach(clearInterval); _intervals = []; },

    async _join() {
      const input = document.getElementById('dash-join-input');
      const id = input?.value?.trim();
      if (!id || id.length !== 16) return Toast.error('Network ID must be 16 hex characters');
      try {
        await api.post(`/local/networks/${id}`, {});
        Toast.success('Joined ' + id);
        if (input) input.value = '';
        refresh();
      } catch(e) { Toast.error(e.message); }
    },

    async _leave(id) {
      if (!await Modal.confirm(`Leave network ${id}?`, { danger: true })) return;
      try { await api.delete(`/local/networks/${id}`); Toast.success('Left ' + id); refresh(); }
      catch(e) { Toast.error(e.message); }
    },

    async _installZt(btn) {
      if (btn) { btn.disabled = true; btn.textContent = 'Installing…'; }
      try {
        const res = await api.post('/system/zt-install', {});
        if (res.status === 'installed' || res.status === 'already_installed') {
          Toast.success('ZeroTier installed — refreshing…');
          setTimeout(refresh, 1500);
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
