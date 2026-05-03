const SettingsGlobalPage = (() => {
  let _origHost, _origPort;

  return {
    async init() {
      document.getElementById('content').innerHTML = Utils.pageLoading();
      let cfg;
      try { cfg = await api.get('/settings/config'); State.set('config', cfg); }
      catch(e) { document.getElementById('content').innerHTML = `<div class="page"><div class="banner banner-danger">❌ ${e.message}</div></div>`; return; }

      _origHost = cfg.server?.host; _origPort = cfg.server?.port;
      document.getElementById('content').innerHTML = `<div class="page">
        <div class="page-header">
          <h1 class="page-title">Global Settings</h1>
          <button class="btn btn-primary" onclick="SettingsGlobalPage._save()">Save</button>
        </div>

        <div class="settings-section">
          <div class="settings-section-title">Server</div>
          <div class="banner banner-warn">⚠️ Changing host or port requires a restart of ztnet-box.</div>
          <div class="field mt"><label class="field-label">Listen Host</label>
            <input class="input" id="s-host" value="${cfg.server?.host||'127.0.0.1'}" style="max-width:240px"></div>
          <div class="field"><label class="field-label">Listen Port</label>
            <input class="input" id="s-port" type="number" min="1" max="65535" value="${cfg.server?.port||3000}" style="max-width:120px"></div>
        </div>

        <div class="settings-section">
          <div class="settings-section-title">ZeroTier Local Service</div>
          <div class="field"><label class="field-label">API URL</label>
            <input class="input" id="s-zt-url" value="${cfg.zerotier?.local?.api_url||'http://127.0.0.1:9993'}"></div>
          <div class="field"><label class="field-label">Token File Path</label>
            <input class="input" id="s-zt-token" value="${cfg.zerotier?.local?.token_file||'/var/lib/zerotier-one/authtoken.secret'}"></div>
        </div>

        <div class="settings-section">
          <div class="settings-section-title">Metrics</div>
          <div class="toggle-wrap">
            <div><div class="toggle-label">Enable Metrics Collection</div></div>
            <label class="toggle">
              <input type="checkbox" id="s-metrics-en" ${cfg.metrics?.enabled?'checked':''}
                onchange="document.getElementById('metrics-opts').style.display=this.checked?'block':'none'">
              <div class="toggle-track"></div><div class="toggle-thumb"></div></label>
          </div>
          <div id="metrics-opts" style="display:${cfg.metrics?.enabled?'block':'none'}">
            <div class="field mt"><label class="field-label">Prometheus URL</label>
              <input class="input" id="s-metrics-url" value="${cfg.metrics?.prometheus_url||'http://127.0.0.1:9993/metrics'}"></div>
            <div class="field"><label class="field-label">Poll Interval (seconds)</label>
              <input class="input" id="s-metrics-interval" type="number" min="1" max="60" value="${cfg.metrics?.poll_interval_seconds||5}" style="max-width:100px"></div>
            <div class="field"><label class="field-label">Metrics Token File</label>
              <input class="input" id="s-metrics-token" value="${cfg.metrics?.metricstoken_file||'/var/lib/zerotier-one/metricstoken.secret'}">
              <div class="text-dim text-sm">Path to <code>metricstoken.secret</code> used for ZeroTier metrics endpoint auth.</div></div>
          </div>
        </div>

        <div class="settings-section">
          <div class="settings-section-title">Exit Node</div>
          <div class="toggle-wrap">
            <div><div class="toggle-label">Prefer nftables over iptables</div></div>
            <label class="toggle">
              <input type="checkbox" id="s-nft" ${cfg.exitnode?.nftables_preferred?'checked':''}>
              <div class="toggle-track"></div><div class="toggle-thumb"></div></label>
          </div>
        </div>
      </div>`;
    },

    async _save() {
      const host = document.getElementById('s-host')?.value?.trim();
      const port = parseInt(document.getElementById('s-port')?.value);
      if (!host) return Toast.error('Host is required');
      if (!port || port < 1 || port > 65535) return Toast.error('Port must be 1–65535');
      const body = {
        server: { host, port },
        zerotier_local: {
          api_url:    document.getElementById('s-zt-url')?.value?.trim(),
          token_file: document.getElementById('s-zt-token')?.value?.trim(),
        },
        metrics: {
          enabled:               document.getElementById('s-metrics-en')?.checked,
          prometheus_url:        document.getElementById('s-metrics-url')?.value?.trim(),
          poll_interval_seconds: parseInt(document.getElementById('s-metrics-interval')?.value)||5,
          metricstoken_file:     document.getElementById('s-metrics-token')?.value?.trim() || null,
        },
        exitnode: { nftables_preferred: document.getElementById('s-nft')?.checked },
      };
      try {
        await api.put('/settings/config', body);
        const serverChanged = host !== _origHost || port !== _origPort;
        Toast.success(serverChanged ? 'Saved. Restart ztnet-box for server changes.' : 'Settings saved.');
        _origHost = host; _origPort = port;
      } catch(e) { errToast(e); }
    },
  };
})();
