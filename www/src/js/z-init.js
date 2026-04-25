// App initialization — runs after all modules are defined.
// MUST be the last JS file loaded (prefixed z- for sort order).

// ── Sidebar toggle (mobile) ────────────────────────────────────────────────────
function toggleSidebar() {
  const sb = document.getElementById('sidebar');
  const ov = document.getElementById('sidebar-overlay');
  const open = sb.classList.toggle('open');
  ov.classList.toggle('visible', open);
}
function closeSidebar() {
  document.getElementById('sidebar').classList.remove('open');
  document.getElementById('sidebar-overlay').classList.remove('visible');
}
document.querySelectorAll('.nav-item').forEach(el => {
  el.addEventListener('click', () => { if (window.innerWidth <= 768) closeSidebar(); });
});

// ── Nav group (collapsible sections) ──────────────────────────────────────────
const NavGroup = (() => {
  const KEY = 'navgroup-state';
  function _load() { try { return JSON.parse(localStorage.getItem(KEY) || '{}'); } catch { return {}; } }
  function _save(s) { try { localStorage.setItem(KEY, JSON.stringify(s)); } catch {} }

  function toggle(id) {
    const el = document.getElementById(id);
    if (!el) return;
    const collapsed = el.classList.toggle('collapsed');
    const s = _load(); s[id] = collapsed; _save(s);
  }
  function expand(id) {
    const el = document.getElementById(id);
    if (!el || !el.classList.contains('collapsed')) return;
    el.classList.remove('collapsed');
    const s = _load(); s[id] = false; _save(s);
  }
  function init() {
    const state = _load();
    document.querySelectorAll('.nav-group').forEach(g => {
      const collapsed = g.id in state ? state[g.id] : true; // default collapsed
      if (collapsed) g.classList.add('collapsed');
    });
  }
  return { toggle, expand, init };
})();

// ── Mobile bar title ───────────────────────────────────────────────────────────
const _routeTitles = {
  '/dashboard':            'Dashboard',
  '/networks':             'Networks',
  '/controllers/networks': 'Controllers',
  '/exitnode':             'Exit Node',
  '/physnet':              'Phys Routing',
  '/bridge':               'L2 Bridge',
  '/relay':                'TCP Relay',
  '/settings/global':      'Global Settings',
  '/settings/ztnode':      'ZeroTier Node',
  '/settings/roots':       'Root Servers',
  '/settings/tokens':      'API Tokens',
};
function _updateMobileTitle(path) {
  const el = document.querySelector('.mobile-bar-title');
  if (!el) return;
  const title = _routeTitles[path]
    || Object.entries(_routeTitles).find(([k]) => path.startsWith(k + '/'))?.[1]
    || 'ZTNetwork Panel';
  el.textContent = title;
}
window.addEventListener('hashchange', () => _updateMobileTitle(location.hash.slice(1) || '/dashboard'));

// ── Register routes ────────────────────────────────────────────────────────────
// NOTE: /peers removed — peers are displayed on the Dashboard.
Router.on('/dashboard',                      () => { DashboardPage.init(); return DashboardPage; });
Router.on('/networks',                       () => { NetworksPage.init(); });
Router.on('/networks/:id',                   (p) => NetworkDetailPage.init(p));
Router.on('/controllers/networks',           () => { CtrlNetworksPage.init(); });
Router.on('/controllers/members/:id',        (p) => { CtrlMembersPage.init(p); });
Router.on('/controllers/config/:id',         (p) => { CtrlConfigPage.init(p); });
Router.on('/exitnode',                       () => { ExitnodePage.init(); });
Router.on('/physnet',                        () => { PhysnetPage.init(); });
Router.on('/bridge',                         () => { BridgePage.init(); });
Router.on('/relay',                          () => { RelayPage.init(); });
Router.on('/settings/global',               () => { SettingsGlobalPage.init(); });
Router.on('/settings/ztnode',               () => { SettingsZtNodePage.init(); });
Router.on('/settings/roots',               () => { SettingsRootsPage.init(); });
Router.on('/settings/tokens',              () => { SettingsTokensPage.init(); });

Router.start();
NavGroup.init();
_updateMobileTitle(location.hash.slice(1) || '/dashboard');
LogPanel.init();

// ── Sidebar status indicators (30s background poll) ───────────────────────────
async function _refreshSidebarStatus() {
  try {
    const node = await api.get('/local/status');
    const dot  = document.getElementById('sidebar-zt-status');
    if (dot) {
      dot.className = 'sidebar-zt-dot ' + (node?.online ? 'online' : 'offline');
      dot.title = 'ZeroTier: ' + (node?.online ? 'Online' : 'Offline');
    }
  } catch {
    const dot = document.getElementById('sidebar-zt-status');
    if (dot) { dot.className = 'sidebar-zt-dot offline'; dot.title = 'ZeroTier: unreachable'; }
  }
  const checks = [
    { id: 'nav-exitnode-badge', url: '/exitnode/status', fn: s => !!s?.enabled,                    group: 'nav-group-gateway' },
    { id: 'nav-physnet-badge',  url: '/physnet/status',  fn: s => !!s?.enabled,                    group: 'nav-group-gateway' },
    { id: 'nav-bridge-badge',   url: '/bridge/status',   fn: s => !!s?.enabled,                    group: 'nav-group-gateway' },
    { id: 'nav-relay-badge',    url: '/relay/status',    fn: s => !!(s?.local?.force_tcp_relay || s?.remote), group: 'nav-group-gateway' },
  ];
  let anyGatewayActive = false;
  for (const c of checks) {
    try {
      const s = await api.get(c.url);
      const active = c.fn(s);
      const el = document.getElementById(c.id);
      if (el) el.style.display = active ? 'block' : 'none';
      if (active) anyGatewayActive = true;
    } catch {}
  }
  if (anyGatewayActive) NavGroup.expand('nav-group-gateway');
}
_refreshSidebarStatus();
setInterval(_refreshSidebarStatus, 30000);
