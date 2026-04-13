const SettingsZtNodePage = (() => {
  let _conf = null;
  let _warnings = [];

  // ── Init ────────────────────────────────────────────────────────────────────

  async function init() {
    document.getElementById('content').innerHTML =
      '<div class="page"><div class="loading-row"><div class="spinner"></div> Loading...</div></div>';
    try {
      _conf = await api.get('/local/config');
      render();
    } catch (e) {
      document.getElementById('content').innerHTML =
        `<div class="page"><div class="banner banner-danger">❌ ${e.message}</div></div>`;
    }
  }

  // ── Render ──────────────────────────────────────────────────────────────────

  function render() {
    const s = _conf?.settings || {};
    const warningBanners = _warnings.length
      ? _warnings
          .map(w => `<div class="banner banner-warn mt-sm">⚠️ ${Utils.esc(w)}</div>`)
          .join('')
      : '';

    document.getElementById('content').innerHTML = `<div class="page">
      <div class="page-header">
        <h1 class="page-title">ZeroTier Node Settings</h1>
        <button class="btn btn-primary" onclick="SettingsZtNodePage._save()">Save</button>
      </div>
      <div class="text-dim text-sm mb">
        These settings are written to ZeroTier's <code>local.conf</code> file.
        Most changes take effect after ZeroTier restarts.
      </div>
      ${warningBanners}

      <div class="settings-section">
        <div class="settings-section-title">Ports</div>
        <div class="field">
          <label class="field-label">Primary UDP Port <span class="text-dim">(default: 9993)</span></label>
          <input class="input" id="zn-primary-port" type="number" min="1" max="65535"
            placeholder="9993" value="${s.primaryPort ?? ''}" style="max-width:120px">
        </div>
        <div class="field">
          <label class="field-label">Secondary UDP Port <span class="text-dim">(0 = disabled)</span></label>
          <input class="input" id="zn-secondary-port" type="number" min="0" max="65535"
            placeholder="0" value="${s.secondaryPort ?? ''}" style="max-width:120px">
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">Port Mapping</div>
        <div class="toggle-wrap">
          <div><div class="toggle-label">UPnP / NAT-PMP Port Mapping</div>
            <div class="text-dim text-sm">Automatically map UDP port on router (default: enabled)</div></div>
          <label class="toggle">
            <input type="checkbox" id="zn-portmap" ${s.portMappingEnabled !== false ? 'checked' : ''}>
            <div class="toggle-track"></div><div class="toggle-thumb"></div>
          </label>
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">TCP Relay</div>
        <div class="toggle-wrap">
          <div><div class="toggle-label">Force TCP Relay</div>
            <div class="text-dim text-sm">Route all ZeroTier traffic through TCP. Impacts performance.</div></div>
          <label class="toggle">
            <input type="checkbox" id="zn-force-tcp" ${s.forceTcpRelay ? 'checked' : ''}>
            <div class="toggle-track"></div><div class="toggle-thumb"></div>
          </label>
        </div>
        <div class="toggle-wrap mt-sm">
          <div><div class="toggle-label">Allow TCP Fallback Relay</div>
            <div class="text-dim text-sm">Use TCP when UDP fails (default: enabled)</div></div>
          <label class="toggle">
            <input type="checkbox" id="zn-allow-tcp" ${s.allowTcpFallbackRelay !== false ? 'checked' : ''}>
            <div class="toggle-track"></div><div class="toggle-thumb"></div>
          </label>
        </div>
        <div class="field mt-sm">
          <label class="field-label">Custom Relay Endpoint <span class="text-dim">(ip/port)</span></label>
          <input class="input" id="zn-relay-ep" placeholder="e.g. 1.2.3.4/443"
            value="${s.tcpFallbackRelay ?? ''}">
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">Bind Addresses</div>
        <div class="text-dim text-sm mb-sm">
          Specific interfaces/IPs to bind to. Leave empty to bind all. One per line.
        </div>
        <textarea class="input" id="zn-bind" rows="3" style="font-family:var(--font-mono);font-size:12px"
          placeholder="192.168.1.10&#10;10.0.0.1">${(s.bind || []).join('\n')}</textarea>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">Interface Blacklist</div>
        <div class="text-dim text-sm mb-sm">
          Interface name prefixes to ignore (e.g. docker, virbr). One per line.
        </div>
        <textarea class="input" id="zn-blacklist" rows="3"
          style="font-family:var(--font-mono);font-size:12px"
          placeholder="docker&#10;virbr&#10;br-">${(s.interfacePrefixBlacklist || []).join('\n')}</textarea>
      </div>

      <div class="settings-section">
        <div class="settings-section-title">Management API Access</div>
        <div class="text-dim text-sm mb-sm">
          IPs/CIDRs allowed to access ZeroTier management API. Default: 127.0.0.1 and ::1. One per line.
        </div>
        <textarea class="input" id="zn-mgmt" rows="3"
          style="font-family:var(--font-mono);font-size:12px"
          placeholder="127.0.0.1&#10;::1">${(s.allowManagementFrom || []).join('\n')}</textarea>
        <div class="text-dim text-sm mt-sm">
          ⚠️ Adding public IPs here exposes the ZeroTier management port to the internet.
        </div>
      </div>
    </div>`;
  }

  // ── Save ────────────────────────────────────────────────────────────────────

  async function _save() {
    const primaryPort = _parsePort('zn-primary-port');
    const secondaryPort = _parsePort('zn-secondary-port');
    const portMappingEnabled = document.getElementById('zn-portmap')?.checked ?? true;
    const forceTcpRelay = document.getElementById('zn-force-tcp')?.checked ?? false;
    const allowTcpFallbackRelay = document.getElementById('zn-allow-tcp')?.checked ?? true;
    const tcpFallbackRelay = document.getElementById('zn-relay-ep')?.value.trim() || null;
    const bind = _parseLines('zn-bind');
    const interfacePrefixBlacklist = _parseLines('zn-blacklist');
    const allowManagementFrom = _parseLines('zn-mgmt');

    const settings = {
      ...(primaryPort != null && { primaryPort }),
      ...(secondaryPort != null && { secondaryPort }),
      portMappingEnabled,
      forceTcpRelay,
      allowTcpFallbackRelay,
      ...(tcpFallbackRelay && { tcpFallbackRelay }),
      ...(bind.length && { bind }),
      ...(interfacePrefixBlacklist.length && { interfacePrefixBlacklist }),
      ...(allowManagementFrom.length && { allowManagementFrom }),
    };

    try {
      const res = await api.put('/local/config', { settings });
      _conf = res;
      _warnings = res.warnings?.map(w => w.message || w) || [];
      render();
      if (_warnings.length) {
        Toast.warn(`Saved with ${_warnings.length} warning(s)`);
      } else {
        Toast.success('ZeroTier node settings saved');
      }
    } catch (e) {
      Toast.error(e.message);
    }
  }

  // ── Helpers ─────────────────────────────────────────────────────────────────

  function _parsePort(id) {
    const val = document.getElementById(id)?.value.trim();
    if (!val) return null;
    const n = parseInt(val, 10);
    return isNaN(n) ? null : n;
  }

  function _parseLines(id) {
    return (document.getElementById(id)?.value || '')
      .split('\n')
      .map(l => l.trim())
      .filter(Boolean);
  }

  return { init, _save };
})();
