const RelayPage = (() => {
  let _data = {};

  async function load() {
    const status = await api.get('/relay/status').catch(() => null);
    _data.status = status;
    render();
  }

  function render() {
    const { status } = _data;
    const el = document.getElementById('content');
    const local = status?.local || {};
    const remote = status?.remote || null;

    const localCard = `
      <div class="section">
        <div class="section-title">Local Configuration</div>
        <div class="card">
          <div class="field">
            <label class="field-label" style="display:flex;align-items:center;gap:.5rem;">
              <input type="checkbox" id="rel-force" ${local.force_tcp_relay ? 'checked' : ''}>
              Force TCP Relay
            </label>
            <div class="text-dim text-sm">Route all ZeroTier traffic through TCP. Impacts performance.</div>
          </div>
          <div class="field">
            <label class="field-label">TCP Fallback Relay <span class="text-dim">(ip/port)</span></label>
            <input class="input" id="rel-endpoint" placeholder="e.g. 1.2.3.4/443" value="${local.tcp_fallback_relay || ''}">
            <div class="text-dim text-sm">Custom relay endpoint. Leave empty to use ZeroTier's default roots.</div>
          </div>
          <button class="btn btn-primary mt-sm" onclick="RelayPage._saveLocal()">Save Local Config</button>
        </div>
      </div>`;

    const remoteStatus = remote ? `
      <div class="card mt">
        <div class="card-header">
          <div class="card-title">Remote Relay</div>
          <span class="badge ${remote.reachable ? 'badge-success' : 'badge-warn'}">${remote.reachable === true ? 'Reachable' : remote.reachable === false ? 'Unreachable' : 'Unknown'}</span>
        </div>
        <div class="detail-kv">
          <span class="k">Host</span><span class="v mono">${remote.host}</span>
          <span class="k">Port</span><span class="v mono">${remote.port}</span>
          <span class="k">Deployed</span><span class="v">${remote.deployed_at ? new Date(remote.deployed_at).toLocaleString() : '—'}</span>
        </div>
        <div class="mt-sm" style="display:flex;gap:.5rem;">
          <button class="btn btn-sm btn-secondary" onclick="RelayPage._verify()">Verify</button>
          <button class="btn btn-sm btn-danger" onclick="RelayPage._removeRemote()">Remove</button>
        </div>
      </div>` : '';

    const deployCard = `
      <div class="section">
        <div class="section-title">Deploy Remote Relay (pylon)</div>
        <div class="card">
          <div class="text-dim text-sm mb-sm">Deploy a <code>zerotier/pylon</code> reflect container via SSH on a remote VPS.</div>
          <div class="field">
            <label class="field-label">Host</label>
            <input class="input" id="dep-host" placeholder="1.2.3.4"></div>
          <div class="field">
            <label class="field-label">SSH Port</label>
            <input class="input" id="dep-sshport" value="22" style="width:80px"></div>
          <div class="field">
            <label class="field-label">SSH User</label>
            <input class="input" id="dep-user" value="root" style="width:120px"></div>
          <div class="field">
            <label class="field-label">Password <span class="text-dim">(or use key)</span></label>
            <input class="input" id="dep-pass" type="password" placeholder="SSH password"></div>
          <div class="field">
            <label class="field-label">Private Key Path <span class="text-dim">(local path, optional)</span></label>
            <input class="input" id="dep-key" placeholder="/home/user/.ssh/id_ed25519"></div>
          <div class="field">
            <label class="field-label">Pylon Port</label>
            <input class="input" id="dep-port" value="443" style="width:80px"></div>
          <div class="field">
            <label class="field-label" style="display:flex;align-items:center;gap:.5rem;">
              <input type="checkbox" id="dep-ufw" checked>
              Stop UFW on remote
            </label>
            <div class="text-dim text-sm">Prevents iptables conflicts with Docker.</div>
          </div>
          <button class="btn btn-primary mt-sm" onclick="RelayPage._deploy()">Deploy</button>
          ${remoteStatus}
        </div>
      </div>`;

    el.innerHTML = `<div class="page">
      <div class="page-header"><h1 class="page-title">TCP Relay</h1></div>
      <div class="banner banner-info mb">
        ℹ️ TCP relay is a fallback when UDP is blocked. Use a custom relay only when ZeroTier's
        built-in roots cannot reach your nodes. Deploy pylon on a VPS with a public IP.
      </div>
      ${localCard}
      ${deployCard}
    </div>`;
  }

  return {
    async init() {
      document.getElementById('content').innerHTML =
        '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading...</div></div>';
      await load();
    },
    async _saveLocal() {
      const force = document.getElementById('rel-force')?.checked || false;
      const ep = document.getElementById('rel-endpoint')?.value.trim() || null;
      try {
        const res = await api.put('/relay/local', { force_tcp_relay: force, tcp_fallback_relay: ep });
        if (res.warnings?.length) res.warnings.forEach(w => Toast.warn(w));
        Toast.success('Local relay config saved');
        load();
      } catch (e) { Toast.error(e.message); }
    },
    async _deploy() {
      const host     = document.getElementById('dep-host')?.value.trim();
      const ssh_port = parseInt(document.getElementById('dep-sshport')?.value) || 22;
      const ssh_user = document.getElementById('dep-user')?.value.trim() || 'root';
      const password = document.getElementById('dep-pass')?.value || null;
      const key_path = document.getElementById('dep-key')?.value.trim() || null;
      const pylon_port = parseInt(document.getElementById('dep-port')?.value) || 443;
      const stop_ufw = document.getElementById('dep-ufw')?.checked ?? true;
      if (!host) return Toast.error('Host is required');
      if (!password && !key_path) return Toast.error('Provide a password or key path');
      if (!await Modal.confirm(
        `Deploy pylon relay on <b>${host}:${pylon_port}</b>?<br>
        <small>SSH will be used. Ensure host is reachable.</small>`)) return;
      const btn = document.querySelector('[onclick="RelayPage._deploy()"]');
      if (btn) { btn.disabled = true; btn.textContent = 'Deploying…'; }
      try {
        await api.post('/relay/deploy', { host, ssh_port, ssh_user, password, key_path, pylon_port, stop_ufw });
        Toast.success(`Relay deployed on ${host}:${pylon_port}`);
        load();
      } catch (e) {
        Toast.error(e.message);
      } finally {
        if (btn) { btn.disabled = false; btn.textContent = 'Deploy'; }
      }
    },
    async _verify() {
      try {
        const res = await api.get('/relay/verify');
        Toast[res.reachable ? 'success' : 'warn'](
          res.reachable ? `Relay reachable at ${res.host}:${res.port}` : 'Relay not reachable'
        );
        load();
      } catch (e) { Toast.error(e.message); }
    },
    async _removeRemote() {
      if (!await Modal.confirm('Remove remote relay?', { danger: true })) return;
      const host = _data.status?.remote?.host;
      const pass = await Modal.prompt?.('SSH password or leave empty to use key:') ?? null;
      try {
        await api.post('/relay/remote', { host: host || '', password: pass || null });
        Toast.success('Remote relay removed');
        load();
      } catch (e) { Toast.error(e.message); }
    },
  };
})();
