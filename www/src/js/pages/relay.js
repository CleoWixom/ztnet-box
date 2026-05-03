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
            <label class="field-label">Private Key Path <span class="text-dim">(local path)</span></label>
            <input class="input" id="dep-key" placeholder="/home/user/.ssh/id_ed25519">
            <div class="text-dim text-sm">Key-based auth only. Ensure the key's public part is in <code>~/.ssh/authorized_keys</code> on the remote host.</div>
          </div>
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
      <div class="page-header">
        <h1 class="page-title">TCP Relay (Pylon)</h1>
        <button class="btn btn-ghost btn-sm" onclick="this.closest('.page').querySelector('.help-box').classList.toggle('hidden')" title="Show/hide help">? Help</button>
      </div>
      <div class="help-box hidden card mb">
        <div class="card-title mb-sm">What is TCP Relay?</div>
        <p class="text-sm text-dim mb-sm">ZeroTier normally uses UDP for peer-to-peer traffic. When UDP is blocked (strict firewalls, corporate networks, CGNATs), nodes can fall back to TCP relay through a <strong>Pylon</strong> — a lightweight relay server. Traffic is encrypted end-to-end; the relay cannot read it.</p>
        <div class="card-title mb-sm mt">When do you need it?</div>
        <ul class="text-sm text-dim mb-sm" style="padding-left:1.2rem;line-height:1.8">
          <li>Nodes show high latency or "RELAY" status in peer list</li>
          <li>Behind firewalls that block all UDP outbound</li>
          <li>On networks where only port 443/TCP is allowed</li>
        </ul>
        <div class="card-title mb-sm mt">How to set up</div>
        <ol class="text-sm text-dim" style="padding-left:1.2rem;line-height:1.8">
          <li>Deploy <strong>Pylon</strong> on a VPS with a public IP and open TCP port (e.g. 443)</li>
          <li>In <em>Local Configuration</em> below: optionally force TCP for testing</li>
          <li>Enter the Pylon endpoint (<code>ip/port</code>) as <em>TCP Fallback Relay</em></li>
          <li>Save — ZeroTier will use this relay when direct UDP paths fail</li>
          <li>Or use <em>Deploy Pylon</em> to install and manage Pylon on this machine via SSH</li>
        </ol>
        <div class="text-sm text-dim mt-sm">⚠️ <em>Force TCP Relay</em> routes ALL traffic through TCP — only enable for debugging, it degrades performance.</div>
      </div>
      ${localCard}
      ${deployCard}
    </div>`;
  }

  return {
    async init() {
      document.getElementById('content').innerHTML =
        Utils.pageLoading();
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
      const key_path = document.getElementById('dep-key')?.value.trim() || null;
      const pylon_port = parseInt(document.getElementById('dep-port')?.value) || 443;
      const stop_ufw = document.getElementById('dep-ufw')?.checked ?? true;
      if (!host) return Toast.error('Host is required');
      if (!await Modal.confirm(
        `Deploy pylon relay on <b>${host}:${pylon_port}</b>?<br>
        <small>SSH will be used. Ensure host is reachable and your key is in authorized_keys.</small>`)) return;
      const btn = document.querySelector('[onclick="RelayPage._deploy()"]');
      if (btn) { btn.disabled = true; btn.textContent = 'Deploying…'; }
      try {
        await api.post('/relay/deploy', { host, ssh_port, ssh_user, key_path, pylon_port, stop_ufw });
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
      try {
        await api.post('/relay/remote', { host: host || '' });
        Toast.success('Remote relay removed');
        load();
      } catch (e) { Toast.error(e.message); }
    },
  };
})();
