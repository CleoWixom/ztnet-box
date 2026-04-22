// App initialization — runs after all modules are defined.
// Extracted from shell.html by build.rs pipeline refactor.
// MUST be the last JS file loaded (prefixed z- for sort order).

// ── Sidebar toggle (mobile) ───────────────────────────────────────────────────
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
// Close sidebar on route change (mobile UX)
document.querySelectorAll('.nav-item').forEach(el => {
  el.addEventListener('click', () => { if (window.innerWidth <= 768) closeSidebar(); });
});

// ── Mobile bar title — updates on navigation ─────────────────────────────────
const _routeTitles = {
  '/dashboard':            'Dashboard',
  '/peers':                'Peers',
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
    || 'ZeroBox';
  el.textContent = title;
}
window.addEventListener('hashchange', () => _updateMobileTitle(location.hash.slice(1) || '/dashboard'));

// Register all routes
Router.on('/dashboard',                      () => { DashboardPage.init(); return DashboardPage; });
Router.on('/peers',                          () => { PeersPage.init(); return PeersPage; });
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
LogPanel.init();

// ── Sidebar status indicators ─────────────────────────────────────────────────
async function _refreshSidebarStatus() {
  try {
    const node = await api.get('/local/status');
    const dot  = document.getElementById('sidebar-zt-status');
    if (dot) { dot.className = 'sidebar-zt-dot ' + (node?.online ? 'online' : 'offline'); }
  } catch(e) {
    const dot = document.getElementById('sidebar-zt-status');
    if (dot) dot.className = 'sidebar-zt-dot offline';
  }
  const checks = [
    { id: 'nav-exitnode-badge', url: '/exitnode/status',  fn: s => s?.enabled },
    { id: 'nav-physnet-badge',  url: '/physnet/status',   fn: s => s?.enabled },
    { id: 'nav-bridge-badge',   url: '/bridge/status',    fn: s => s?.enabled },
    { id: 'nav-relay-badge',    url: '/relay/status',     fn: s => s?.local?.force_tcp_relay || !!s?.remote },
  ];
  for (const c of checks) {
    try {
      const s  = await api.get(c.url);
      const el = document.getElementById(c.id);
      if (el) el.style.display = c.fn(s) ? 'block' : 'none';
    } catch(e) {}
  }
}
_refreshSidebarStatus();
setInterval(_refreshSidebarStatus, 30000);
