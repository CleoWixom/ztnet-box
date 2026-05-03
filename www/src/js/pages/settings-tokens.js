const SettingsTokensPage = (() => {
  let _tokens = [];
  let _verifyResult = null;

  function renderTokenCard(t) {
    const isActive = t.is_active;
    return `<div class="token-card${isActive?' active-token':''}">
      <div class="token-info">
        <div class="token-name">${t.name} ${isActive?'<span class="badge badge-primary">Active</span>':''}</div>
        <div class="token-masked">${t.masked_token}</div>
        <div class="text-sm text-dim mt-sm">
          ${t.rate_limit==='paid'?'<span class="badge badge-success">Paid</span>':'<span class="badge badge-muted">Free</span>'}
          Added ${t.created_at?new Date(t.created_at).toLocaleDateString():''}
        </div>
      </div>
      <div class="token-actions">
        ${!isActive?`<button class="btn btn-ghost btn-sm" onclick="SettingsTokensPage._activate('${t.id}')">Set Active</button>`:''}
        <button class="btn btn-danger btn-sm" onclick="SettingsTokensPage._delete('${t.id}','${t.name}')">Delete</button>
      </div>
    </div>`;
  }

  function render() {
    const hasActive = _tokens.some(t=>t.is_active);
    document.getElementById('content').innerHTML = `<div class="page">
      <div class="page-header">
        <h1 class="page-title">API Tokens</h1>
        <button class="btn btn-primary" onclick="SettingsTokensPage._showAddForm()">+ Add Token</button>
      </div>
      ${!hasActive&&_tokens.length?`<div class="banner banner-warn">⚠️ No active token — ZeroTier Central API is unavailable.</div>`:''}
      ${!_tokens.length?`<div class="empty-state"><div class="empty-state-icon">🔑</div>
        <h3>No API tokens</h3>
        <p>Add a ZeroTier Central API token to manage networks and members via the Central API.</p>
        <button class="btn btn-primary mt" onclick="SettingsTokensPage._showAddForm()">Add your first token</button>
      </div>`:_tokens.map(renderTokenCard).join('')}
      <div id="add-form" style="display:none" class="card mt">
        <div class="card-header"><div class="card-title">Add Token</div></div>
        <div class="field"><label class="field-label">Name</label>
          <input class="input" id="tok-name" placeholder="e.g. My ZeroTier Account"></div>
        <div class="field"><label class="field-label">API Token</label>
          <input class="input" id="tok-value" type="password" placeholder="Paste your Central API token"></div>
        <div id="verify-result"></div>
        <div style="display:flex;gap:8px;margin-top:var(--gap)">
          <button class="btn btn-ghost" onclick="SettingsTokensPage._verify()">🔍 Verify</button>
          <button class="btn btn-primary" id="add-btn" disabled onclick="SettingsTokensPage._add()">Add Token</button>
          <button class="btn btn-ghost" onclick="document.getElementById('add-form').style.display='none'">Cancel</button>
        </div>
      </div>
    </div>`;
  }

  return {
    async init() {
      document.getElementById('content').innerHTML = Utils.pageLoading();
      try { _tokens = await api.get('/settings/tokens'); } catch(e) { _tokens = []; }
      render();
    },
    _showAddForm() {
      render();
      const el = document.getElementById('add-form');
      if (el) el.style.display = 'block';
      _verifyResult = null;
    },
    async _verify() {
      const token = document.getElementById('tok-value')?.value?.trim();
      if (!token) return Toast.error('Enter a token to verify');
      const btn = document.getElementById('add-btn');
      try {
        const res = await api.post('/settings/tokens/validate', { token });
        if (res.valid) {
          _verifyResult = res;
          document.getElementById('verify-result').innerHTML = `
            <div class="banner banner-info mt">
              ✅ Valid — ${res.account_status?.display_name||''} (${res.account_status?.email||'—'})
              <span class="badge ${res.rate_limit==='paid'?'badge-success':'badge-muted'} ml">${res.rate_limit}</span>
            </div>`;
          if (btn) btn.disabled = false;
        } else {
          document.getElementById('verify-result').innerHTML = `<div class="banner banner-danger mt">❌ ${res.error||'Invalid token'}</div>`;
          if (btn) btn.disabled = true;
        }
      } catch(e) { errToast(e); }
    },
    async _add() {
      const name = document.getElementById('tok-name')?.value?.trim();
      const token = document.getElementById('tok-value')?.value?.trim();
      if (!name) return Toast.error('Name is required');
      if (!token) return Toast.error('Token is required');
      try {
        await api.post('/settings/tokens', { name, token });
        Toast.success('Token added');
        _tokens = await api.get('/settings/tokens');
        render();
      } catch(e) { errToast(e); }
    },
    async _activate(id) {
      try { await api.post(`/settings/tokens/${id}/activate`); Toast.success('Active token updated'); _tokens = await api.get('/settings/tokens'); render(); }
      catch(e) { errToast(e); }
    },
    async _delete(id, name) {
      if (!await Modal.confirm(`Delete token "${name}"?`, {danger:true})) return;
      try { await api.delete(`/settings/tokens/${id}`); Toast.success('Deleted'); _tokens = await api.get('/settings/tokens'); render(); }
      catch(e) { errToast(e); }
    },
  };
})();
