// CRM Quality Inspector - frontend v0.4
// Features: KPI management, auto-scoring, predictive risk queue, manual override

const $ = (s) => document.querySelector(s);
const $$ = (s) => Array.from(document.querySelectorAll(s));

const TOKEN_KEY = 'crm_qi_token';
const USER_KEY = 'crm_qi_user';
const CACHE_KEY = 'crm_qi_cache_v1';
const CACHE_TTL = 60_000;
const LANG_KEY = 'crm_qi_lang';
let currentLang = localStorage.getItem(LANG_KEY) || 'fa';

const State = {
  token: localStorage.getItem(TOKEN_KEY) || null,
  user: JSON.parse(localStorage.getItem(USER_KEY) || 'null'),
  agents: [], customers: [], interactions: [], rubrics: [],
  scores: {}, issues: [], recommendations: [], kpis: [],
  dashboard: null, agentsAvg: {},
  loaded: { agents: false, customers: false, interactions: false, issues: false, rubrics: false, kpis: false, dashboard: false, rec: false, coaching: false, calibration: false, audit: false },
  trendChart: null,
  scoreChart: null,
  agentChart: null,
  page: { interactions: 1, issues: 1, agents: 1, customers: 1 },
  pageSize: 10,
  interactionsTotal: 0,
  interactionsTotalPages: 1,
  agentsTotal: 0,
  agentsTotalPages: 1,
  customersTotal: 0,
  customersTotalPages: 1,
  issuesTotal: 0,
  issuesTotalPages: 1,
};

function setToken(t, u) {
  State.token = t; State.user = u;
  if (t) { localStorage.setItem(TOKEN_KEY, t); localStorage.setItem(USER_KEY, JSON.stringify(u)); }
  else { localStorage.removeItem(TOKEN_KEY); localStorage.removeItem(USER_KEY); }
  invalidateCache();
}

function invalidateCache() { localStorage.removeItem(CACHE_KEY); }

function cacheGet(key) {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    if (!raw) return null;
    const obj = JSON.parse(raw);
    const e = obj[key];
    if (!e || Date.now() - e.t > CACHE_TTL) return null;
    return e.v;
  } catch { return null; }
}

function cacheSet(key, v) {
  try {
    const raw = localStorage.getItem(CACHE_KEY) || '{}';
    const obj = JSON.parse(raw);
    obj[key] = { t: Date.now(), v };
    localStorage.setItem(CACHE_KEY, JSON.stringify(obj));
  } catch {}
}

async function api(url, opts = {}) {
  const headers = { 'Content-Type': 'application/json', ...(opts.headers || {}) };
  if (State.token) headers['Authorization'] = `Bearer ${State.token}`;
  const r = await fetch('/api' + url, { ...opts, headers });
  // Handle 401 globally: token expired or invalid → force re-login with a toast
  if (r.status === 401) {
    // Avoid loops: only redirect if we still think we're logged in
    if (State.token) {
      const reason = url === '/auth/login' ? t('loginError') : t('toastLoginExpired');
      logout(reason);
    }
    throw new Error(t('toastLoginExpired'));
  }
  const text = await r.text();
  if (!text) {
    if (r.ok) return null;
    throw new Error(t('toastDataLoaded'));
  }
  let j;
  try { j = JSON.parse(text); } catch { throw new Error(t('invalidResponse')); }
  if (!j.success) throw new Error(j.error || t('toastDataLoaded'));
  return j.data;
}

function esc(s) { return String(s ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c])); }

// Loading overlay helpers
let _loadingCount = 0;
function showLoading(text) {
  _loadingCount++;
  const ov = $('#loadingOverlay');
  if (!ov) return;
  if (text) $('#loadingText').textContent = text;
  else $('#loadingText').textContent = t('loadingAllLabel');
  ov.hidden = false;
}
function hideLoading() {
  _loadingCount = Math.max(0, _loadingCount - 1);
  if (_loadingCount === 0) {
    const ov = $('#loadingOverlay');
    if (ov) ov.hidden = true;
  }
}
async function withLoading(text, fn) {
  showLoading(text);
  try { return await fn(); }
  finally { hideLoading(); }
}
function fmtDate(iso) {
  if (!iso) return '-';
  try { return new Date(iso).toLocaleString('fa-IR', { dateStyle: 'short', timeStyle: 'short' }); }
  catch { return iso; }
}
function scorePill(v, critical) {
  if (v == null) return `<span class="pill pill-muted">${t('scoreNotEvaluated')}</span>`;
  const cls = critical ? 'pill-bad' : v >= 85 ? 'pill-good' : v >= 70 ? 'pill-info' : v >= 60 ? 'pill-warn' : 'pill-bad';
  return `<span class="pill ${cls}">${Number(v).toFixed(1)}${critical ? ' ⚠' : ''}</span>`;
}
function sevPill(s) {
  const map = { [t('severityCritical')]: 'pill-bad', [t('severityHigh')]: 'pill-warn', [t('severityMedium')]: 'pill-info', [t('severityLow')]: 'pill-muted' };
  return `<span class="pill ${map[s] || 'pill-muted'}">${esc(s)}</span>`;
}
function statusPill(s) {
  return s === t('isOpenStatus') ? `<span class="pill pill-warn">${t('isOpenStatus')}</span>` : `<span class="pill pill-good">${t('isClosedStatus')}</span>`;
}
function priorityPill(p) {
  const map = { [t('priorityHigh')]: 'pill-bad', [t('priorityMedium')]: 'pill-warn', [t('priorityLow')]: 'pill-info' };
  return `<span class="pill ${map[p] || 'pill-muted'}">${t('priorityLabel')} ${esc(p)}</span>`;
}
function toast(msg, type = 'success') {
  const toastEl = $('#toast');
  toastEl.textContent = msg;
  toastEl.className = 'toast show ' + type;
  setTimeout(() => toastEl.classList.remove('show'), 3500);
}

// ============ Login ============
$('#loginForm').addEventListener('submit', async (e) => {
  e.preventDefault();
  const username = $('#loginUser').value.trim();
  const password = $('#loginPass').value;
  const err = $('#loginError');
  err.classList.remove('show');
  try {
    const data = await withLoading(t('enteringLoginLabel'), () => api('/auth/login', { method: 'POST', body: JSON.stringify({ username, password }) }));
    setToken(data.token, { username: data.username, is_admin: data.is_admin });
    enterApp();
  } catch (ex) {
    err.textContent = ex.message;
    err.classList.add('show');
  }
});

function logout(message) {
  setToken(null, null);
  State.agents = []; State.customers = []; State.interactions = [];
  State.rubrics = []; State.scores = {}; State.issues = [];
  State.recommendations = []; State.kpis = []; State.dashboard = null;
  Object.keys(State.loaded).forEach(k => State.loaded[k] = false);
  $('#loginScreen').classList.remove('hidden');
  $('#appShell').classList.add('hidden');
  // Show a clear message about WHY the user was logged out
  if (message) {
    setTimeout(() => toast(message, 'warning'), 100);
  }
}

// ============ App ============
async function enterApp() {
  $('#loginScreen').classList.add('hidden');
  $('#appShell').classList.remove('hidden');
  $('#userBadge').textContent = State.user?.username || '';
  $('#topbarUser').textContent = State.user?.username || '';
  // Show audit nav only for admins
  const auditNav = $('#auditNavItem');
  if (auditNav) auditNav.style.display = State.user?.is_admin ? '' : 'none';
  const mobileAuditNav = $('#mobileAuditNavItem');
  if (mobileAuditNav) mobileAuditNav.style.display = State.user?.is_admin ? '' : 'none';
  await loadDashboard();
  switchTab('dashboard');
  // Eagerly pre-fetch the customer risk page so the first tab switch
  // shows data immediately instead of a loading spinner.
  loadCustomerRisk(true);
}

async function loadDashboard() {
  if (State.loaded.dashboard) { renderDashboard(); return; }
  try {
    // Performance: load dashboard summary + bulk scores + agents in parallel.
    // Agents is small (just names for chart labels), so we fetch it now too.
    // Full pagination of agents/customers/issues happens on tab switch.
    const [dash, scoresList, agentsData, interactionsData] = await Promise.all([
      withLoading(t('toastLoadDashboard'), () => api('/reports/dashboard')),
      api('/scores'),
      api('/agents?page=1&limit=1000'),
      api('/interactions?page=1&limit=1000')  // load for agent chart lookups
    ]);
    State.dashboard = dash;
    // Bulk-load all scores into State.scores by interaction_id (used by trend/agent charts)
    State.scores = {};
    (scoresList || []).forEach(s => { if (s && s.interaction_id) State.scores[s.interaction_id] = s; });
    // Populate agents for chart labels
    State.agents = (agentsData && agentsData.items) || [];
    State.agentsTotal = (agentsData && agentsData.total) || State.agents.length;
    State.agentsTotalPages = (agentsData && agentsData.total_pages) || 1;
    State.loaded.agents = true;
    // Populate interactions so renderAgentChart can find them
    State.interactions = (interactionsData && interactionsData.items) || [];
    State.interactionsTotal = (interactionsData && interactionsData.total) || State.interactions.length;
    State.interactionsTotalPages = (interactionsData && interactionsData.total_pages) || 1;
    State.loaded.interactions = true;
    State.loaded.dashboard = true;
    cacheSet(t('dashboard'), State.dashboard);
    cacheSet(t('agents'), State.agents);
    // Render dashboard immediately — all 3 charts have data
    renderDashboard();
    // Background: refresh interactions when user clicks the tab (loads page 1 of paginated view)
    setTimeout(async () => {
      if (!State.loaded.interactions) {
        await loadInteractions();
      }
    }, 100);
  } catch (e) {
    toast(t('toastErrorDashboard') + e.message, 'error');
  }
}

async function loadAgents(force = false) {
  const pageSize = State.pageSize || 10;
  const page = State.page.agents || 1;
  if (!force && State.loaded.agents) { renderAgents(); return; }
  const c = cacheGet(t('agents')); if (c) { State.agents = c; State.loaded.agents = true; State.agentsTotal = c.length; State.agentsTotalPages = 1; renderAgents(); return; }
  const data = await api(`/agents?page=${page}&limit=${pageSize}`);
  State.agents = data.items || [];
  State.agentsTotal = data.total;
  State.agentsTotalPages = data.total_pages;
  State.loaded.agents = true;
  cacheSet(t('agents'), State.agents);
  renderAgents();
}
async function loadCustomers(force = false) {
  const pageSize = State.pageSize || 10;
  const page = State.page.customers || 1;
  if (!force && State.loaded.customers) { renderCustomers(); return; }
  const c = cacheGet(t('customers')); if (c) { State.customers = c; State.loaded.customers = true; State.customersTotal = c.length; State.customersTotalPages = 1; renderCustomers(); return; }
  const data = await api(`/customers?page=${page}&limit=${pageSize}`);
  State.customers = data.items || [];
  State.customersTotal = data.total;
  State.customersTotalPages = data.total_pages;
  State.loaded.customers = true;
  cacheSet(t('customers'), State.customers);
  renderCustomers();
}
async function loadInteractions(force = false) {
  if (!force && State.loaded.interactions && State.interactions.length) { renderInteractions(); return; }
  const pageSize = State.pageSize || 10;
  const page = State.page.interactions || 1;
  const data = await api(`/interactions?page=${page}&limit=${pageSize}`);
  State.interactions = data.items || [];
  State.interactionsTotal = data.total;
  State.interactionsTotalPages = data.total_pages;
  State.loaded.interactions = true;
  loadAllScoresLazy();
  renderInteractions();
}
async function loadAllScoresLazy() {
  // Skip scores we already have (bulk-loaded by loadDashboard via GET /scores)
  const missing = State.interactions.filter(it => !State.scores[it.id]);
  if (missing.length === 0) {
    renderInteractions();
    if (document.getElementById('trendChart')) renderTrendChart();
    if (State.loaded.agents && document.getElementById('agentChart') && State.agentChart === null) {
      renderAgentChart();
    }
    return;
  }
  const promises = missing.map(it =>
    api('/scoring/' + it.id).then(s => { if (s) State.scores[it.id] = s; }).catch(() => null)
  );
  await Promise.all(promises);
  renderInteractions();
  if (document.getElementById('trendChart')) renderTrendChart();
  if (State.loaded.agents && document.getElementById('agentChart') && State.agentChart === null) {
    renderAgentChart();
  }
}
async function loadIssues(force = false) {
  const pageSize = State.pageSize || 10;
  const page = State.page.issues || 1;
  if (!force && State.loaded.issues) { renderIssues(); return; }
  // Build filter query
  const params = new URLSearchParams({ page, limit: pageSize });
  const status = $('#iStatus')?.value;
  const severity = $('#iSeverity')?.value;
  if (status) params.set(t('status'), status);
  if (severity) params.set(t('severity'), severity);
  const data = await api(`/issues?${params}`);
  State.issues = data.items || [];
  State.issuesTotal = data.total;
  State.issuesTotalPages = data.total_pages;
  State.loaded.issues = true;
  renderIssues();
}
async function loadKpis() {
  if (State.loaded.kpis) { renderKpis(); return; }
  State.kpis = await api('/kpis'); State.loaded.kpis = true;
  renderKpis();
}
async function loadRecommendations() {
  if (State.loaded.rec) { renderRecommendations(); return; }
  State.recommendations = await withLoading(t('toastLoadAnalysis'), () => api('/recommendations')); State.loaded.rec = true;
  renderRecommendations();
}

// ============ Customer Risk Score ============
async function loadCustomerRisk(force = false) {
  if (!force && State.loaded.risk) { renderCustomerRisk(); return; }
  State.customerRisk = await withLoading(t('toastLoadRisk'), () => api('/customers/risk'));
  State.loaded.risk = true;
  renderCustomerRisk();
}

function renderCustomerRisk() {
  const list = State.customerRisk || [];
  const high = list.filter(c => c.level === t('riskHigh').split(' ')[0]).length;
  const med  = list.filter(c => c.level === t('riskMed').split(' ')[0]).length;
  const low  = list.filter(c => c.level === t('riskLow').split(' ')[0]).length;

  // KPIs (matches dashboard's .kpi/.kpi-value style)
  $('#riskKpiGrid').innerHTML = `
    <div class="kpi">
      <div class="kpi-label">${t('riskHigh')}</div>
      <div class="kpi-value kpi-red">${high}</div>
      <div class="kpi-sub" style="font-size:11px;color:var(--text-muted);margin-top:4px">${t('riskHighSub')}</div>
    </div>
    <div class="kpi">
      <div class="kpi-label">${t('riskMed')}</div>
      <div class="kpi-value kpi-amber">${med}</div>
      <div class="kpi-sub" style="font-size:11px;color:var(--text-muted);margin-top:4px">${t('riskMedSub')}</div>
    </div>
    <div class="kpi">
      <div class="kpi-label">${t('riskLow')}</div>
      <div class="kpi-value kpi-green">${low}</div>
      <div class="kpi-sub" style="font-size:11px;color:var(--text-muted);margin-top:4px">${t('riskLowSub')}</div>
    </div>
    <div class="kpi">
      <div class="kpi-label">${t('totalCustomersLabel')}</div>
      <div class="kpi-value">${list.length}</div>
      <div class="kpi-sub" style="font-size:11px;color:var(--text-muted);margin-top:4px">${t('withRiskScoreLabel')}</div>
    </div>
  `;

  // Table
  const tbody = $('#riskTable tbody');
  if (list.length === 0) {
    tbody.innerHTML = `<tr><td colspan="9" style="text-align:center;color:var(--text-muted);padding:24px">داده‌ای موجود نیست</td></tr>`;
    return;
  }
  tbody.innerHTML = list.map(c => {
    const score = (c.risk_score || 0).toFixed(1);
    const avg = c.avg_score != null ? c.avg_score.toFixed(1) : '—';
    const lastDate = c.last_interaction_at
      ? new Date(c.last_interaction_at).toLocaleDateString('fa-IR')
      : '—';
    const factors = (c.factors || []).slice(0, 2).map(f =>
      `<div class="factor-chip" title="${esc(f.reason)}">${esc(f.label)} +${f.points.toFixed(0)}</div>`
    ).join('');
    const factorDetail = (c.factors || []).length > 2
      ? `<span class="factor-chip" style="background:var(--surface-2)">+${c.factors.length - 2} ${t('moreLabel')}</span>`
      : '';
    const levelClass = c.level === t('riskHigh').split(' ')[0] ? 'risk-high' : c.level === t('riskMed').split(' ')[0] ? 'risk-med' : 'risk-low';
    const actionClass = c.level === t('riskHigh').split(' ')[0] ? 'badge-red' : c.level === t('riskMed').split(' ')[0] ? 'badge-amber' : 'badge-green';
    return `
      <tr>
        <td><span class="risk-dot ${levelClass}" title="${score}"></span></td>
        <td><strong>${esc(c.customer_name)}</strong><br><span style="font-size:11px;color:var(--text-muted)">${lastDate}</span></td>
        <td><strong style="font-size:18px;color:var(--${c.level === t('riskHigh').split(' ')[0] ? 'red' : c.level === t('riskMed').split(' ')[0] ? 'amber' : 'green'})">${score}</strong></td>
        <td><span class="badge ${actionClass}">${esc(c.level)}</span></td>
        <td>${c.scored_interactions || 0} / ${c.total_interactions || 0}</td>
        <td>${c.open_issues > 0 ? `<span class="badge badge-red">${c.open_issues}</span>` : '—'}</td>
        <td>${avg}</td>
        <td>${esc(c.recommended_action)}</td>
        <td><div style="display:flex;flex-wrap:wrap;gap:4px">${factors}${factorDetail}</div></td>
      </tr>
    `;
  }).join('');
}

// ============ Language Switcher ============
function toggleLanguage() {
  // Toggle and apply - show OPPOSITE language button
  currentLang = currentLang === 'fa' ? 'en' : 'fa';
  applyLanguage(currentLang);
}

function applyLanguage(lang) {
  localStorage.setItem(LANG_KEY, lang);
  document.documentElement.dir = lang === 'fa' ? 'rtl' : 'ltr';
  document.documentElement.lang = lang;
  // Update lang buttons - show OPPOSITE so user clicks to SWITCH
  const btns = [$('#langBtn'), $('#langBtnLogin')].filter(Boolean);
  btns.forEach(btn => btn.textContent = lang === 'fa' ? '🇬🇧 EN' : '🇮🇷 FA');
  // Update all data-i18n elements
  $$('[data-i18n]').forEach(el => {
    const key = el.getAttribute('data-i18n');
    if (I18N[lang][key] !== undefined) el.textContent = I18N[lang][key];
  });
  // Update page title
  document.title = I18N[lang]['pageTitle'] || 'Quality Inspector';
}

function switchTab(tab) {
  $$('.page').forEach(p => p.classList.add('hidden'));
  $$('.nav-item').forEach(n => n.classList.remove('active'));
  $('#page-' + tab).classList.remove('hidden');
  $(`.nav-item[data-tab="${tab}"]`)?.classList.add('active');
  $('#topbarTitle').innerHTML = ({
    dashboard: t('navDashboard'), interactions: t('navInteractions'), agents: t('navAgents'),
    customers: t('navCustomers'), risk: t('navRisk'), recommendations: t('navRecommendations'),
    issues: t('navIssues'), calibration: t('navCalibration'),
    rubrics: t('navRubrics'), report: t('navReport'), coaching: t('navCoaching'),
    users: t('navUsers'), audit: t('navAudit')
  }[tab] || tab);
  if (tab === 'dashboard') loadDashboard();
  else if (tab === 'interactions') loadInteractions();
  else if (tab === 'agents') loadAgents();
  else if (tab === 'customers') loadCustomers();
  else if (tab === 'issues') loadIssues();
  else if (tab === 'coaching') loadCoaching();
  else if (tab === 'calibration') loadCalibration();
  else if (tab === 'rubrics') loadKpis();
  else if (tab === 'recommendations') loadRecommendations();
  else if (tab === 'risk') loadCustomerRisk();
  else if (tab === 'report') loadAgents().then(populateReportAgents);
  else if (tab === 'users') loadUsers();
  else if (tab === 'audit') loadAudit(1, true);
}

$$('.nav-item').forEach(n => n.addEventListener('click', () => switchTab(n.dataset.tab)));
$('#logoutBtn').addEventListener('click', logout);

// ============ Mobile Navigation ============
const mobileMenuBtn = $('#mobileMenuBtn');
const mobileNav = $('#mobileNav');
const mobileNavOverlay = $('#mobileNavOverlay');
const mobileNavClose = $('#mobileNavClose');
const mobileLogoutBtn = $('#mobileLogoutBtn');

if (mobileMenuBtn) {
  mobileMenuBtn.addEventListener('click', () => {
    mobileNav.classList.add('open');
    mobileNavOverlay.classList.add('active');
  });
}

function closeMobileNav() {
  mobileNav.classList.remove('open');
  mobileNavOverlay.classList.remove('active');
}

if (mobileNavClose) {
  mobileNavClose.addEventListener('click', closeMobileNav);
}
if (mobileNavOverlay) {
  mobileNavOverlay.addEventListener('click', closeMobileNav);
}
if (mobileLogoutBtn) {
  mobileLogoutBtn.addEventListener('click', () => {
    closeMobileNav();
    logout();
  });
}

// Sync mobile nav items with desktop navigation
$$('.mobile-nav-item').forEach(item => {
  item.addEventListener('click', () => {
    switchTab(item.dataset.tab);
    closeMobileNav();
  });
});

// ============ Dashboard ============
function renderDashboard() {
  const d = State.dashboard || {};
  const kpis = [
    { label: t('kpiAgentsLabel'), value: d.agent_count, cls: 'primary' },
    { label: t('kpiCustomersLabel'), value: d.customer_count, cls: 'info' },
    { label: t('kpiInteractionsLabel'), value: d.interaction_count, cls: 'primary' },
    { label: t('kpiCoverageLabel'), value: (d.coverage || 0).toFixed(1) + '%', cls: 'info' },
    { label: t('kpiAvgScoreLabel'), value: (d.average_score || 0).toFixed(1), cls: d.average_score >= 80 ? 'success' : d.average_score >= 60 ? 'warning' : 'danger' },
    { label: t('kpiOpenIssuesLabel'), value: d.open_issues, cls: d.open_issues > 0 ? 'warning' : 'success' },
    { label: t('criticalFailLabel'), value: d.critical_failures, cls: d.critical_failures > 0 ? 'danger' : 'success' },
    { label: t('kpiQualityGradeLabel'), value: d.quality_grade, cls: 'primary' },
  ];
  $('#kpiGrid').innerHTML = kpis.map(k =>
    `<div class="kpi"><div class="kpi-label">${k.label}</div><div class="kpi-value ${k.cls}">${k.value}</div></div>`
  ).join('');
  $('#coverageBar').innerHTML = `
    <div style="display:flex;justify-content:space-between;margin-bottom:8px">
      <span style="color:var(--text-muted)">${t('scoredCount')(d.scored_count || 0, d.interaction_count || 0)}</span>
      <strong>${(d.coverage || 0).toFixed(1)}%</strong>
    </div>
    <div class="bar-track"><div class="bar-fill" style="width:${Math.min(100, d.coverage || 0)}%"></div></div>
  `;
  renderTrendChart();
  renderScoreChart(d);
  renderAgentChart();
}

function renderTrendChart() {
  const canvas = $('#trendChart');
  if (!canvas) return;
  if (State.trendChart) { State.trendChart.destroy(); State.trendChart = null; }
  const scores = Object.values(State.scores);
  if (scores.length < 2) {
    const ctx = canvas.getContext('2d');
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = '#8a92a6'; ctx.font = '14px Tahoma';
    ctx.fillText(t('chartNoData'), 10, 30);
    return;
  }
  const sorted = scores.slice().sort((a, b) => new Date(a.created_at) - new Date(b.created_at));
  const labels = sorted.map(s => fmtDate(s.created_at));
  const data = sorted.map(s => s.overall_score);
  State.trendChart = new Chart(canvas, {
    type: 'line',
    data: { labels, datasets: [{
      label: t('avgQualityScore'),
      data,
      borderColor: '#6366f1',
      backgroundColor: 'rgba(99,102,241,0.15)',
      fill: true, tension: 0.3, pointRadius: 4, pointBackgroundColor: '#8b5cf6',
    }] },
    options: {
      responsive: true, maintainAspectRatio: false,
      animation: { duration: 0 },
      plugins: { legend: { labels: { color: '#e6e8ee' } } },
      scales: {
        x: { ticks: { color: '#8a92a6' }, grid: { color: '#2a2f3d' } },
        y: { min: 0, max: 100, ticks: { color: '#8a92a6' }, grid: { color: '#2a2f3d' } },
      },
    },
  });
}

function renderScoreChart(d) {
  const canvas = $('#scoreChart');
  if (!canvas) return;
  if (State.scoreChart) { State.scoreChart.destroy(); State.scoreChart = null; }
  const scores = Object.values(State.scores);
  const healthy = scores.filter(s => s.overall_score >= 80).length;
  const improvement = scores.filter(s => s.overall_score >= 60 && s.overall_score < 80).length;
  const critical = scores.filter(s => s.overall_score < 60).length;
  const ctx = canvas.getContext('2d');
  State.scoreChart = new Chart(ctx, {
    type: 'doughnut',
    data: {
      labels: [t('criticalLabel')],
      datasets: [{
        data: [healthy, improvement, critical],
        backgroundColor: ['#22c55e', '#f59e0b', '#ef4444'],
        borderWidth: 0
      }]
    },
    options: {
      responsive: true, maintainAspectRatio: false,
      plugins: { legend: { position: 'bottom', labels: { color: '#e6e8ee', padding: 12 } } }
    }
  });
}

function renderAgentChart() {
  const canvas = $('#agentChart');
  if (!canvas) return;
  if (State.agentChart) { State.agentChart.destroy(); State.agentChart = null; }
  const agents = State.agents.filter(a => a.active);
  const labels = agents.map(a => a.name);
  const data = labels.map(name => {
    const scores = Object.values(State.scores).filter(s => {
      const interaction = State.interactions.find(i => i.id === s.interaction_id);
      return interaction && interaction.agent_id === agents.find(a => a.name === name)?.id;
    });
    return scores.length ? scores.reduce((a, b) => a + b.overall_score, 0) / scores.length : 0;
  });
  const ctx = canvas.getContext('2d');
  State.agentChart = new Chart(ctx, {
    type: 'bar',
    data: {
      labels,
      datasets: [{
        label: t('kpiAvgScoreText'),
        data,
        backgroundColor: '#6366f1',
        borderRadius: 4
      }]
    },
    options: {
      responsive: true, maintainAspectRatio: false,
      plugins: { legend: { display: false } },
      scales: {
        x: { ticks: { color: '#8a92a6' }, grid: { display: false } },
        y: { min: 0, max: 100, ticks: { color: '#8a92a6' }, grid: { color: '#2a2f3d' } }
      }
    }
  });
}
function renderPagination(selector, total, page, totalPages, onChange) {
  const el = document.querySelector(selector);
  if (!el) return;
  if (total === 0) { el.innerHTML = `<span style="color:var(--text-muted);font-size:12px">${t('noRecordLabel')}</span>`; return; }
  const pageSize = State.pageSize || 10;
  const start = (page - 1) * pageSize + 1;
  const end = Math.min(page * pageSize, total);
  const btn = (label, p, dis) =>
    `<button class="btn btn-sm" data-pg="${p}" ${dis ? 'disabled style="opacity:.4;cursor:not-allowed"' : ''}>${label}</button>`;
  const sizeOptions = [10, 20, 50, 100];
  const sizeSelect = sizeOptions.includes(pageSize)
    ? `<select class="pager-size-select" style="padding:4px 8px;border-radius:6px;background:var(--surface-2);color:var(--foreground);border:1px solid var(--border)">
         ${sizeOptions.map(n => `<option value="${n}" ${n === pageSize ? 'selected' : ''}>${n}</option>`).join('')}
         <option value="custom" ${!sizeOptions.includes(pageSize) ? 'selected' : ''}>${t('customPageSize')}</option>
       </select>`
    : `<select class="pager-size-select" style="padding:4px 8px;border-radius:6px;background:var(--surface-2);color:var(--foreground);border:1px solid var(--border)">
         ${sizeOptions.map(n => `<option value="${n}">${n}</option>`).join('')}
         <option value="custom" selected>${t('customPageSize')}</option>
       </select>`;
  const customInput = !sizeOptions.includes(pageSize)
    ? `<input type="number" class="pager-size-custom" min="1" max="1000" value="${pageSize}" style="width:70px;padding:4px 8px;border-radius:6px;background:var(--surface-2);color:var(--foreground);border:1px solid var(--border);margin-right:4px" placeholder='${t("pageSizePlaceholder")}' />`
    : '';
  el.innerHTML =
    `<div class="pager-bar" style="display:flex;align-items:center;gap:12px;flex-wrap:wrap">
       <span class="pager-info">${t('pageInfoLabel')(start, end, total, page, totalPages)}</span>
       <div class="pager-buttons" style="display:flex;gap:4px">
         ${btn(t('pagerFirst'), 1, page === 1)}
           ${btn(t('pagerPrev'), page - 1, page === 1)}
           ${btn(t('pagerNext'), page + 1, page === totalPages)}
           ${btn(t('pagerLast'), totalPages, page === totalPages)}
       </div>
       <div class="pager-size" style="display:flex;align-items:center;gap:4px;margin-right:auto">
         <span style="font-size:12px;color:var(--text-muted)">${t('pageSizeLabel')}</span>
         ${sizeSelect}
         ${customInput}
       </div>
     </div>`;
  el.querySelectorAll('button[data-pg]').forEach(b => {
    b.addEventListener('click', () => {
      const p = parseInt(b.dataset.pg, 10);
      if (p >= 1 && p <= totalPages) onChange(p);
    });
  });
  const sel = el.querySelector('.pager-size-select');
  if (sel) {
    sel.addEventListener('change', (e) => {
      const v = e.target.value;
      if (v === 'custom') {
        // Show prompt for custom value
        const n = parseInt(prompt(t('pageSizePrompt')), 10) || pageSize;
        if (n >= 1 && n <= 1000) {
          State.pageSize = n;
          State.page.interactions = 1;
          onChange(1);
        } else {
          // Revert select
          e.target.value = sizeOptions.includes(pageSize) ? pageSize : 'custom';
        }
      } else {
        State.pageSize = parseInt(v, 10);
        State.page.interactions = 1;
        onChange(1);
      }
    });
  }
}

// ============ Interactions ============
function renderInteractions() {
  const tbody = $('#interactionsTable tbody');
  if (!tbody) return;
  const search = ($('#fSearch')?.value || '').toLowerCase();
  const channel = $('#fChannel')?.value || '';
  const agentId = $('#fAgent')?.value || '';
  const status = $('#fStatus')?.value || '';
  const pageSize = State.pageSize || 10;

  const sel = $('#fAgent');
  if (sel && sel.options.length <= 1 && State.agents.length) {
    sel.innerHTML = '<option value="">' + t('allAgentsOpt') + '</option>' + State.agents
      .filter(a => a.active).map(a => `<option value="${a.id}">${esc(a.name)}</option>`).join('');
  }

  // If server didn't paginate (returned all rows), do client-side filtering
  // If server paginated (interactionsTotal set), we already have only the current page
  const isServerPaginated = State.interactionsTotal > 0 && State.interactionsTotal !== State.interactions.length;
  let rows = State.interactions;
  if (!isServerPaginated) {
    rows = State.interactions.slice();
    if (search) rows = rows.filter(i => (i.subject + ' ' + i.transcript + ' ' + (i.tags||[]).join(' ')).toLowerCase().includes(search));
    if (channel) rows = rows.filter(i => i.channel === channel);
    if (agentId) rows = rows.filter(i => i.agent_id === agentId);
    if (status === 'scored') rows = rows.filter(i => State.scores[i.id]);
    if (status === 'unscored') rows = rows.filter(i => !State.scores[i.id]);
    rows.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));
  } else {
    // Server-paginated: just sort the current page; filters would need server support
    rows = State.interactions.slice().sort((a, b) => new Date(b.created_at) - new Date(a.created_at));
  }

  // Use server-provided total if available, else compute
  const total = State.interactionsTotal || rows.length;
  const totalPages = State.interactionsTotalPages || Math.max(1, Math.ceil(rows.length / pageSize));
  const page = State.page.interactions || 1;
  // When server-paginated, rows are already just the current page so start=0
  const start = isServerPaginated ? 0 : (page - 1) * pageSize;
  const pageRows = isServerPaginated ? rows : rows.slice(start, start + pageSize);

  tbody.innerHTML = pageRows.map(i => {
    const s = State.scores[i.id];
    const agent = State.agents.find(a => a.id === i.agent_id);
    const customer = State.customers.find(c => c.id === i.customer_id);
    return `<tr>
      <td data-label='${t("colDate")}'>${fmtDate(i.created_at)}</td>
      <td data-label='${t("colAgent")}'>${esc(agent?.name || '-')}</td>
      <td data-label='${t("colCustomer")}'>${esc(customer?.name || '-')}</td>
      <td data-label='${t("colChannel")}'><span class="pill pill-muted">${esc(i.channel)}</span></td>
      <td data-label='${t("colSubject")}'><b>${esc(i.subject)}</b><div style="color:var(--text-muted);font-size:12px;margin-top:2px">${esc((i.transcript || '').slice(0, 80))}${(i.transcript || '').length > 80 ? '...' : ''}</div></td>
      <td data-label='${t("colScore")}'>${scorePill(s?.overall_score, s?.critical_fail)}</td>
      <td class="row-actions" data-label='${t("colActions")}'>
        <button class="btn btn-sm btn-primary" data-auto="${i.id}">${s ? t('reviewBtn') : t('autoScoreBtn')}</button>
        <button class="btn btn-sm" data-view="${i.id}">${t('viewInterBtn')}</button>
      </td>
    </tr>`;
  }).join('') || `<tr><td colspan="7" style="text-align:center;padding:40px;color:var(--text-muted)">${t('emptyTable')}</td></tr>`;

  tbody.querySelectorAll('[data-auto]').forEach(b => b.addEventListener('click', () => openAutoScore(b.dataset.auto)));
  tbody.querySelectorAll('[data-view]').forEach(b => b.addEventListener('click', () => openView(b.dataset.view)));

  // Render pagination footer
  renderPagination('#interactionsPager', total, page, totalPages, (newPage) => {
    State.page.interactions = newPage;
    loadInteractions(true);
  });
}

$('#fSearch')?.addEventListener('input', renderInteractions);
$('#fChannel')?.addEventListener('change', renderInteractions);
$('#fAgent')?.addEventListener('change', renderInteractions);
$('#fStatus')?.addEventListener('change', renderInteractions);
$('#exportCsvBtn')?.addEventListener('click', exportCsv);

function exportCsv() {
  if (!State.interactions.length) { toast(t('toastAuthRequired'), 'error'); return; }
  const rows = [t('exportCsvHeaders')];
  for (const i of State.interactions) {
    const s = State.scores[i.id];
    const agent = State.agents.find(a => a.id === i.agent_id);
    const customer = State.customers.find(c => c.id === i.customer_id);
    rows.push([i.id, fmtDate(i.created_at), agent?.name || '', customer?.name || '',
      i.channel, i.subject, s ? s.overall_score : '', s ? s.level : '',
      s ? (s.critical_fail ? t('yesText') : t('noText')) : '', s?.notes || '']);
  }
  const csv = '\uFEFF' + rows.map(r => r.map(c => `"${String(c).replace(/"/g, '""')}"`).join(',')).join('\n');
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement(t('a'));
  a.href = url; a.download = `interactions-${new Date().toISOString().slice(0,10)}.csv`;
  a.click(); URL.revokeObjectURL(url);
  toast(t('csvDownloadedToast'));
}

// ============ Auto-Score Modal ============
async function openAutoScore(id) {
  const interaction = State.interactions.find(i => i.id === id);
  if (!interaction) return;
  if (State.kpis.length === 0) await loadKpis();
  const customer = State.customers.find(c => c.id === interaction.customer_id);
  const agent = State.agents.find(a => a.id === interaction.agent_id);

  // First show a preview/measure
  openModal(t('autoScoreTitle'), `
    <div style="background:var(--surface-2);padding:12px;border-radius:8px;margin-bottom:14px">
      <b>${esc(interaction.subject)}</b>
      <div style="color:var(--text-muted);font-size:12px;margin-top:4px">${esc(interaction.transcript.slice(0, 300))}${interaction.transcript.length > 300 ? '…' : ''}</div>
    </div>
    <div style="text-align:center;padding:20px">
      <div class="spinner" style="display:inline-block;width:24px;height:24px;border:3px solid var(--border);border-top-color:var(--primary);border-radius:50%;animation:spin 1s linear infinite"></div>
      <p style="color:var(--text-muted);margin-top:12px">${t('autoScoreProgress')}</p>
    </div>
  `);
  document.head.insertAdjacentHTML('beforeend', '<style>@keyframes spin{to{transform:rotate(360deg)}}</style>');

  try {
    const res = await api('/kpis/measure/' + id);
    const measurements = res.measurements || [];
    const overall = res.overall_score;
    const level = res.level;
    const cf = res.critical_fail;
    const scoreColor = cf ? 'var(--danger)' : overall >= 85 ? 'var(--success)' : overall >= 60 ? 'var(--warning)' : 'var(--danger)';

    openModal(t('autoScoreTitle'), `
      <div style="background:var(--surface-2);padding:12px;border-radius:8px;margin-bottom:14px">
        <b>${esc(interaction.subject)}</b>
        <div style="color:var(--text-muted);font-size:12px;margin-top:4px">
          ${esc(agent?.name || '-')} | ${esc(customer?.name || '-')}
        </div>
      </div>

      <div style="text-align:center;padding:20px;background:var(--surface-2);border-radius:10px;margin-bottom:16px">
        <div style="font-size:48px;font-weight:800;color:${scoreColor}">${overall.toFixed(1)}</div>
        <div style="color:var(--text-muted);font-size:14px;margin-top:4px">${esc(level)}</div>
        ${cf ? `<div class="pill pill-bad" style="margin-top:8px">${t('criticalFailLabel')}</div>` : ''}
      </div>

      <h3 style="margin:0 0 10px;font-size:14px">${t('autoScoreDetailTitle')}</h3>
      <div class="measurements">
        ${measurements.map(m => `
          <div class="meas-item ${m.critical_fail ? 'meas-fail' : ''}">
            <div class="meas-row">
              <span class="meas-name">${esc(m.kpi_name)} ${m.critical ? '<span class="pill pill-bad" style="font-size:10px">بحرانی</span>' : ''}</span>
              <span class="meas-score" style="color:${m.score < 60 ? 'var(--danger)' : m.score >= 85 ? 'var(--success)' : 'var(--warning)'}">${m.score.toFixed(0)}</span>
            </div>
            <div class="meas-bar"><div class="meas-fill" style="width:${Math.min(100, m.score)}%;background:${m.score < 60 ? 'var(--danger)' : m.score >= 85 ? 'var(--success)' : 'var(--primary)'}"></div></div>
            <div class="meas-evidence">${esc(m.evidence)}</div>
          </div>
        `).join('')}
      </div>

      <div class="field" style="margin-top:14px">
        <label>${t('evaluatorNote')}</label>
        <textarea id="autoScoreNotes" placeholder="${t('notesPlaceholder')}"></textarea>
      </div>
    `, `<button class="btn btn-primary" id="saveAutoScore">${t('saveToDashboardBtn')}</button>
        <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);

    $('#saveAutoScore').addEventListener('click', async () => {
      const btn = $('#saveAutoScore');
      btn.disabled = true; btn.textContent = t('savingText');
      try {
        const notes = $('#autoScoreNotes').value.trim();
        const saved = await api('/scoring/auto/' + id, {
          method: 'POST',
          body: JSON.stringify(notes ? { notes } : {}),
        });
        State.scores[id] = saved;
        invalidateCache();
        closeModal();
        renderInteractions();
        renderTrendChart();
        toast(t('autoScoreResultSaved'), saved.overall_score, saved.level);
      } catch (e) {
        toast(e.message, 'error');
        btn.disabled = false; btn.textContent = `${t('saveToDashboardBtn')}`;
      }
    });
  } catch (e) {
    closeModal();
    if (e.message.includes(t('noActiveKpi'))) {
      toast('ابتدا KPI تعریف کنید یا پیش‌فرض‌ها را بارگذاری کنید', 'error');
      switchTab('rubrics');
    } else {
      toast(e.message, 'error');
    }
  }
}

async function openView(id) {
  const i = State.interactions.find(x => x.id === id);
  if (!i) return;
  await loadAgents(); await loadCustomers();
  const agent = State.agents.find(a => a.id === i.agent_id);
  const customer = State.customers.find(c => c.id === i.customer_id);
  const s = State.scores[id];
  openModal(t('interactionDetailTitle'), `
    <div class="field"><b>${t('colAgent')}:</b> ${esc(agent?.name || '-')} <span class="pill pill-info">${esc(agent?.department || '')}</span></div>
    <div class="field"><b>${t('colCustomer')}:</b> ${esc(customer?.name || '-')} <span class="pill pill-muted">${esc(customer?.segment || '')}</span></div>
    <div class="field"><b>${t('colChannel')}:</b> <span class="pill pill-muted">${esc(i.channel)}</span> &nbsp; <b>${t('colDate')}:</b> ${fmtDate(i.created_at)}</div>
    <div class="field"><b>${t('colSubject')}:</b> ${esc(i.subject)}</div>
    <div class="field" style="background:var(--surface-2);padding:12px;border-radius:8px">
      <b style="display:block;margin-bottom:6px">${t('transcriptLabel')}</b>
      <div style="white-space:pre-wrap">${esc(i.transcript)}</div>
    </div>
    ${s ? `
      <div style="margin-top:14px;padding:16px;background:var(--surface-2);border-radius:8px;text-align:center">
        <div style="font-size:36px;font-weight:800;color:${s.critical_fail ? 'var(--danger)' : 'var(--success)'}">${Number(s.overall_score).toFixed(1)}</div>
        <div style="color:var(--text-muted)">${esc(s.level)}</div>
        ${s.critical_fail ? `<div class="pill pill-bad" style="margin-top:8px">${t('criticalFailLabel2')}</div>` : ''}
        ${s.notes ? `<div style="margin-top:10px;text-align:right;color:var(--text-muted);font-size:12px">${esc(s.notes)}</div>` : ''}
        ${s.evaluator ? `<div style="margin-top:6px;color:var(--text-muted);font-size:11px">${t('evaluatorLabel')}: ${esc(s.evaluator)}</div>` : ''}
      </div>
    ` : ''}
    <button class="btn btn-primary" id="fromViewScore">${s ? t('reviewBtn') : t('autoScoreBtn')}</button>
    <button class="btn" data-action="close-modal">${t('closeBtn')}</button>
  `);
  $('#fromViewScore')?.addEventListener('click', () => { closeModal(); openAutoScore(id); });
}

// ============ Recommendations (Risk Queue) ============
function renderRecommendations() {
  const list = $('#recommendationList');
  if (!list) return;
  if (!State.recommendations.length) {
    list.innerHTML = '<div class="list-item" style="text-align:center;color:var(--text-muted);padding:40px">' + t('allInteractionsScored') + '</div>';
    return;
  }
  list.innerHTML = State.recommendations.map(r => {
    const color = r.risk_score >= 70 ? 'var(--danger)' : r.risk_score >= 40 ? 'var(--warning)' : 'var(--info)';
    const reasons = r.reasons || r.factors || [];
    return `<div class="rec-card">
      <div class="rec-header">
        <div>
          <div class="rec-subject">${esc(r.subject)}</div>
          <div style="color:var(--text-muted);font-size:12px;margin-top:2px">
            ${t('expertCustomerChannel')(esc(r.agent_name || '-'), '', '')} | ${t('colCustomer')}: ${esc(r.customer_name || '-')} | ${t('colChannel')}: ${esc(r.channel)}
          </div>
        </div>
        <div style="text-align:center">
          <div class="risk-score" style="color:${color}">${r.risk_score.toFixed(0)}</div>
          <div style="font-size:10px;color:var(--text-muted)">${t('riskLabel')}</div>
          ${priorityPill(r.priority)}
        </div>
      </div>
      <div class="rec-body">
        <div style="margin-bottom:8px"><b>${t('riskFactorsLabel')}</b></div>
        <ul class="factors">
          ${reasons.map(f => {
            if (typeof f === 'string') {
              // API returns simple strings - just show as pill
              return `<li><span class="factor-pill">${esc(f)}</span></li>`;
            } else if (typeof f === 'object' && f !== null) {
              // Fallback for object format (future-proof)
              const pillText = f.label || f.code || '';
              const reasonText = f.reason || '';
              if (!reasonText) {
                return `<li><span class="factor-pill">${esc(pillText)}</span></li>`;
              }
              return `<li><span class="factor-pill">${esc(pillText)}</span> <span style="color:var(--text-muted);font-size:12px">${esc(reasonText)}</span></li>`;
            }
            return '';
          }).join('')}
        </ul>
        <div style="margin-top:8px;padding:8px;background:var(--surface-2);border-radius:6px">
          <b>${t('suggestedActionLabel')}</b> ${esc(r.suggested_action)}
        </div>
        <div style="margin-top:10px">
          <button class="btn btn-sm btn-primary" data-rec-auto="${r.interaction_id}">${t('autoScoreBtn2')}</button>
        </div>
      </div>
    </div>`;
  }).join('');
  list.querySelectorAll('[data-rec-auto]').forEach(b => b.addEventListener('click', () => {
    switchTab('interactions');
    setTimeout(() => openAutoScore(b.dataset.recAuto), 200);
  }));
}

// ============ Agents ============
function renderAgents() {
  const tbody = $('#agentsTable tbody');
  if (!tbody) return;
  tbody.innerHTML = State.agents.map(a => `
    <tr>
      <td data-label='${t("colName")}'><b>${esc(a.name)}</b></td>
      <td data-label='${t("colDepartment")}'><span class="pill pill-info">${esc(a.department)}</span></td>
      <td data-label='${t("colPosition")}'>${esc(a.position)}</td>
      <td>${a.active ? '<span class="pill pill-good">' + t('isActiveAgent') + '</span>' : '<span class="pill pill-muted">' + t('isInactiveAgent') + '</span>'}</td>
      <td class="row-actions" data-label='${t("colActions")}'>
        <button class="btn btn-sm" data-toggle-agent="${a.id}" data-active="${!a.active}">${a.active ? t('isInactiveAgent') : t('isActiveAgent')}</button>
        <button class="btn btn-sm btn-primary" data-agent-report="${a.id}">${t('reportBtn2')}</button>
      </td>
    </tr>
  `).join('') || `<tr><td colspan="5" style="text-align:center;padding:40px;color:var(--text-muted)">${t('emptyAgentMsg')}</td></tr>`;
  tbody.querySelectorAll('[data-toggle-agent]').forEach(b => b.addEventListener('click', () => toggleAgent(b.dataset.toggleAgent, b.dataset.active === 'true')));
  tbody.querySelectorAll('[data-agent-report]').forEach(b => b.addEventListener('click', () => { switchTab('report'); setTimeout(() => { $('#reportAgent').value = b.dataset.agentReport; renderReport(); }, 50); }));
  renderPagination('#agentsPager', State.agentsTotal || 0, State.page.agents, State.agentsTotalPages || 1, (p) => { State.page.agents = p; loadAgents(true); });
}

async function toggleAgent(id, active) {
  try {
    await api('/agents/' + id, { method: 'PATCH', body: JSON.stringify({ active }) });
    State.agents = await api('/agents'); cacheSet(t('agents'), State.agents);
    renderAgents();
    toast(t('toggleToast')(active));
  } catch (e) { toast(e.message, 'error'); }
}

function openNewAgent() {
  openModal(t('newAgentTitle'), `
    <div class="field"><label>${t('agentNameLabel')}</label><input id="aName" placeholder="مثال: علی رضایی"></div>
    <div class="field"><label>${t('agentDeptLabel')}</label><select id="aDept"><option>بانک</option><option>بیمه</option><option>عمومی</option></select></div>
    <div class="field"><label>${t('agentPosLabel')}</label><input id="aPos" placeholder='${t("agentPosPlaceholder")}'></div>
  `, `<button class="btn btn-primary" id="saveAgent">${t('saveBtn')}</button>
      <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);
  $('#saveAgent').addEventListener('click', async () => {
    try {
      await api('/agents', { method: 'POST', body: JSON.stringify({
        name: $('#aName').value.trim(), department: $('#aDept').value, position: $('#aPos').value.trim()
      })});
      State.loaded.agents = false;
      closeModal(); await loadAgents();
      toast(t('agentSavedToast'));
    } catch (e) { toast(e.message, 'error'); }
  });
}

// ============ Customers ============
function renderCustomers() {
  const tbody = $('#customersTable tbody');
  if (!tbody) return;
  tbody.innerHTML = State.customers.map(c => `
    <tr>
      <td><b>${esc(c.name)}</b>${c.notes ? `<div style="color:var(--text-muted);font-size:12px">${esc(c.notes)}</div>` : ''}</td>
      <td>${esc(c.phone)}</td>
      <td><span class="pill pill-info">${esc(c.product_type)}</span></td>
      <td>${esc(c.segment)}</td>
      <td class="row-actions" data-label='${t("colActions")}'>
        <button class="btn btn-sm" data-edit-customer="${c.id}">${t('editBtn')}</button>
        <button class="btn btn-sm" data-del-customer="${c.id}">${t('deleteBtn')}</button>
      </td>
    </tr>
  `).join('') || `<tr><td colspan="5" style="text-align:center;padding:40px;color:var(--text-muted)">${t('emptyCustomerMsg')}</td></tr>`;
  tbody.querySelectorAll('[data-edit-customer]').forEach(b => b.addEventListener('click', () => openEditCustomer(b.dataset.editCustomer)));
  tbody.querySelectorAll('[data-del-customer]').forEach(b => b.addEventListener('click', async () => {
    if (!confirm(t('confirmDelete')(c.name))) return;
    try { await api('/customers/' + b.dataset.delCustomer, { method: 'DELETE' }); State.loaded.customers = false; await loadCustomers(); toast(t('customerDeletedToast')); }
    catch (e) { toast(e.message, 'error'); }
  }));
  renderPagination('#customersPager', State.customersTotal || 0, State.page.customers, State.customersTotalPages || 1, (p) => { State.page.customers = p; loadCustomers(true); });
}

function openEditCustomer(id) {
  const c = State.customers.find(x => x.id === id);
  if (!c) return;
  openModal(t('editCustomerTitle')(c.name), `
    <div class="field"><label>${t('colName')}</label><input id="cName" value="${esc(c.name)}"></div>
    <div class="field"><label>${t('colPhone')}</label><input id="cPhone" value="${esc(c.phone)}"></div>
    <div class="field"><label>${t('colProduct')}</label>
      <select id="cProduct">
        <option ${c.product_type===t('productBank')?'selected':''}>${t('productBank')}</option>
        <option ${c.product_type===t('productInsurance')?'selected':''}>${t('productInsurance')}</option>
        <option ${c.product_type===t('productInvestment')?'selected':''}>${t('productInvestment')}</option>
        <option ${c.product_type===t('productLoan')?'selected':''}>${t('productLoan')}</option>
      </select>
    </div>
    <div class="field"><label>${t('segmentLabel')}</label>
      <select id="cSegment">
        <option ${c.segment===t('segmentNormal')?'selected':''}>${t('segmentNormal')}</option>
        <option ${c.segment===t('segmentImportant')?'selected':''}>${t('segmentImportant')}</option>
        <option ${c.segment==='VIP'?'selected':''}>${t('vipOpt')}</option>
      </select>
    </div>
    <div class="field"><label>${t('notesFieldLabel')}</label><textarea id="cNotes" rows="2">${esc(c.notes || '')}</textarea></div>
  `, `<button class="btn btn-primary" id="saveEditCust">${t('saveBtn')}</button>
      <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);
  $('#saveEditCust').addEventListener('click', async () => {
    try {
      await api('/customers/' + encodeURIComponent(id), {
        method: 'PATCH',
        body: JSON.stringify({
          name: $('#cName').value.trim(),
          phone: $('#cPhone').value.trim(),
          product_type: $('#cProduct').value,
          segment: $('#cSegment').value,
          notes: $('#cNotes').value.trim()
        })
      });
      State.loaded.customers = false;
      closeModal(); await loadCustomers();
      toast(t('customerUpdatedToast'));
    } catch (e) { toast(e.message, 'error'); }
  });
}

function openNewCustomer() {
  openModal(t('newCustomerTitle'), `
    <div class="field"><label>${t('newCustomerName')}</label><input id="cName"></div>
    <div class="field"><label>${t('newCustomerPhone')}</label><input id="cPhone"></div>
    <div class="field"><label>${t('newCustomerProduct')}</label><input id="cProduct" placeholder="مثال: تسهیلات"></div>
    <div class="field"><label>${t('newCustomerSeg')}</label><select id="cSeg"><option>${t('normalOpt')}</option><option>${t('vipOpt')}</option><option>${t('corpOpt')}</option></select></div>
    <div class="field"><label>${t('notesFieldLabel')}</label><textarea id="cNotes"></textarea></div>
  `, `<button class="btn btn-primary" id="saveCustomer">${t('saveBtn')}</button>
      <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);
  $('#saveCustomer').addEventListener('click', async () => {
    try {
      await api('/customers', { method: 'POST', body: JSON.stringify({
        name: $('#cName').value.trim(), phone: $('#cPhone').value.trim(),
        product_type: $('#cProduct').value.trim(), segment: $('#cSeg').value,
        notes: $('#cNotes').value.trim()
      })});
      State.loaded.customers = false;
      closeModal(); await loadCustomers(); toast(t('customerSavedToast'));
    } catch (e) { toast(e.message, 'error'); }
  });
}

// ============ New Interaction ============
function openNewInteraction() {
  if (!State.agents.length || !State.customers.length) {
    Promise.all([loadAgents(), loadCustomers()]).then(openNewInteraction);
    return;
  }
  openModal(t('newInteractionTitle'), `
    <div class="field"><label>${t('colAgent')}</label><select id="iAgent">${State.agents.filter(a => a.active).map(a => `<option value="${a.id}">${esc(a.name)} — ${esc(a.department)}</option>`).join('')}</select></div>
    <div class="field"><label>${t('colCustomer')}</label><select id="iCust">${State.customers.map(c => `<option value="${c.id}">${esc(c.name)} — ${esc(c.product_type)}</option>`).join('')}</select></div>
    <div class="field"><label>${t('colChannel')}</label><select id="iCh"><option>${t('channelPhone')}</option><option>${t('channelInperson')}</option><option>${t('channelEmail')}</option><option>${t('channelChat')}</option><option>${t('channelSms')}</option></select></div>
    <div class="field"><label>${t('colSubject')}</label><input id="iSub"></div>
    <div class="field"><label>متن مکالمه</label><textarea id="iTr" style="min-height:120px" placeholder='${t("transcriptPlaceholder")}'></textarea></div>
  `, `<button class="btn btn-primary" id="saveInteraction">${t('saveBtn')}</button>
      <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);
  $('#saveInteraction').addEventListener('click', async () => {
    try {
      const sub = $('#iSub').value.trim();
      const tr = $('#iTr').value.trim();
      if (!sub || !tr) throw new Error(t('requiredSubjectTranscriptError'));
      await api('/interactions', { method: 'POST', body: JSON.stringify({
        agent_id: $('#iAgent').value, customer_id: $('#iCust').value,
        channel: $('#iCh').value, subject: sub, transcript: tr, tags: []
      })});
      State.loaded.interactions = false;
      State.loaded.dashboard = false;
      State.loaded.rec = false;
      closeModal(); await loadInteractions(); await loadDashboard();
      toast(t('interactionSavedToast'));
    } catch (e) { toast(e.message, 'error'); }
  });
}

// ============ KPIs Management ============
function renderKpis() {
  const list = $('#kpiList');
  if (!list) return;
  if (!State.kpis.length) {
    list.innerHTML = `<div class="card" style="text-align:center;padding:40px;color:var(--text-muted)">
      <div style="font-size:48px;margin-bottom:12px">📊</div>
      <div style="margin-bottom:16px">${t('noKpiDefined')}</div>
      <button class="btn btn-primary" id="emptySeedKpis">${t('seedDefaultKpisBtn')}</button>
    </div>`;
    $('#emptySeedKpis')?.addEventListener('click', seedKpis);
    return;
  }
  const kindLabel = {
    keyword_count: t('kindKeywordCount'),
    keyword_presence: t('kindKeywordPresence'),
    text_length: t('kindTextLength'),
    keyword_ratio: t('kindKeywordRatio'),
    response_time: t('kindResponseTime'),
    manual_range: t('kindManual'),
  };
  list.innerHTML = State.kpis.map(k => `
    <div class="kpi-card ${k.active ? '' : 'kpi-inactive'}">
      <div class="kpi-card-header">
        <div>
          <div class="kpi-card-name">${esc(k.name)} ${k.critical ? '<span class="pill pill-bad" style="font-size:10px">بحرانی</span>' : ''}</div>
          <div class="kpi-card-code"><code>${esc(k.code)}</code></div>
        </div>
        <div style="text-align:left">
          <div style="font-size:11px;color:var(--text-muted)">${kindLabel[k.kind] || k.kind}</div>
          <div style="font-size:20px;font-weight:700;color:var(--primary)">${k.weight}<span style="font-size:12px;color:var(--text-muted)">٪</span></div>
        </div>
      </div>
      <div class="kpi-card-desc">${esc(k.description)}</div>
      ${k.pattern ? `<div style="font-size:11px;color:var(--text-muted);margin:4px 0"><b>${t('kpiPatternLabel')}</b> <code>${esc(k.pattern)}</code></div>` : ''}
      ${k.threshold != null ? `<div style="font-size:11px;color:var(--text-muted);margin:4px 0"><b>${t('kpiThresholdLabel')}</b> ${esc(k.threshold)}</div>` : ''}
      <div class="kpi-card-actions">
        <button class="btn btn-sm" data-toggle-kpi="${k.id}" data-active="${!k.active}">${k.active ? t('isInactiveAgent') : t('isActiveAgent')}</button>
        <button class="btn btn-sm" data-del-kpi="${k.id}">${t('deleteCustLabel')}</button>
      </div>
    </div>
  `).join('');
  list.querySelectorAll('[data-toggle-kpi]').forEach(b => b.addEventListener('click', async () => {
    try {
      const active = b.dataset.active === 'true';
      await api('/kpis/' + b.dataset.toggleKpi, { method: 'PATCH', body: JSON.stringify({ active }) });
      State.kpis = await api('/kpis'); renderKpis();
      toast(t('toggleToast')(active));
    } catch (e) { toast(e.message, 'error'); }
  }));
  list.querySelectorAll('[data-del-kpi]').forEach(b => b.addEventListener('click', async () => {
    if (!confirm(t('deleteKpiConfirm'))) return;
    try {
      await api('/kpis/' + b.dataset.delKpi, { method: 'DELETE' });
      State.kpis = await api('/kpis'); renderKpis();
      toast(t('kpiDeletedToast'));
    } catch (e) { toast(e.message, 'error'); }
  }));
}

async function seedKpis() {
  try {
    const result = await api('/kpis/seed', { method: 'POST' });
    State.kpis = await api('/kpis');
    renderKpis();
    toast(t('kpisLoadedToast')(result.length));
  } catch (e) { toast(e.message, 'error'); }
}

$('#seedKpisBtn')?.addEventListener('click', seedKpis);
$('#newKpiBtn')?.addEventListener('click', openNewKpi);

function openNewKpi() {
  openModal(t('newKpiTitle'), `
  <div class="field"><label>${t('kpiCodeLabel')}</label><input id="kCode" placeholder="${t('kpiCodePlaceholder')}"></div>
  <div class="field"><label>${t('kpiNameLabel')}</label><input id="kName" placeholder="${t('kpiNamePlaceholder')}"></div>
  <div class="field"><label>${t('kpiKindLabel')}</label>
    <select id="kKind">
      <option value="keyword_count">${t('kindKeywordCountOption')}</option>
      <option value="keyword_presence">${t('kindKeywordPresenceOption')}</option>
      <option value="text_length">${t('kindTextLengthOption')}</option>
      <option value="keyword_ratio">${t('kindKeywordRatioOption')}</option>
      <option value="response_time">${t('kindResponseTimeOption')}</option>
      <option value="manual_range">${t('kindManualOption')}</option>
    </select>
  </div>
  <div class="field"><label>${t('kpiDescLabel')}</label><textarea id="kDesc" placeholder="${t('kpiDescPlaceholder')}"></textarea></div>
  <div class="field"><label>${t('kpiPatternLabel')}</label>
    <input id="kPattern" placeholder="${t('kpiPatternPlaceholder')}">
  </div>
  <div class="field"><label>${t('kpiThresholdLabel')}</label>
    <input id="kThreshold" type="number" step="0.1" placeholder="${t('kpiThresholdPlaceholder')}">
  </div>
    <div class="field"><label>${t('kpiWeightLabel')}</label><input id="kWeight" type="number" min="0" max="100" step="1" value="10"></div>
    <label><input type="checkbox" id="kCritical"> ${t('criticalFailNote')}</label>
  `, `<button class="btn btn-primary" id="saveKpi">${t('saveKpiBtn')}</button>
      <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);
  $('#saveKpi').addEventListener('click', async () => {
    const code = $('#kCode').value.trim();
    const name = $('#kName').value.trim();
    const weight = parseFloat($('#kWeight').value);
    if (!code || !name) { toast(t('kpiCodeNameRequired'), 'error'); return; }
    if (isNaN(weight) || weight < 0 || weight > 100) { toast(t('kpiWeightError'), 'error'); return; }
    const th = $('#kThreshold').value.trim();
    const req = {
      code, name,
      kind: $('#kKind').value,
      description: $('#kDesc').value.trim(),
      pattern: $('#kPattern').value.trim() || null,
      threshold: th ? parseFloat(th) : null,
      weight,
      critical: $('#kCritical').checked,
    };
    try {
      await api('/kpis', { method: 'POST', body: JSON.stringify(req) });
      State.kpis = await api('/kpis');
      closeModal(); renderKpis();
      toast(t('kpiAddedToast'));
    } catch (e) { toast(e.message, 'error'); }
  });
}

// ============ Issues ============
function renderIssues() {
  const tbody = $('#issuesTable tbody');
  if (!tbody) return;
  const status = $('#iStatus')?.value || '';
  const sev = $('#iSeverity')?.value || '';
  let rows = State.issues.slice();
  if (status) rows = rows.filter(x => x.status === status);
  if (sev) rows = rows.filter(x => x.severity === sev);
  rows.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));
  tbody.innerHTML = rows.map(x => {
    return `<tr>
      <td>${sevPill(x.severity)}</td>
      <td>${esc(x.category)}</td>
      <td style="max-width:360px">${esc(x.description)}${x.root_cause ? `<div style="color:var(--text-muted);font-size:12px;margin-top:4px"><b>علت:</b> ${esc(x.root_cause)}</div>` : ''}</td>
      <td>${statusPill(x.status)}</td>
      <td>${x.due_at ? fmtDate(x.due_at) : '-'}</td>
      <td class="row-actions" data-label='${t("colActions")}'>
        ${x.status === t('isOpenStatus') ? `<button class="btn btn-sm btn-success" data-resolve-issue="${x.id}">${t('capaBtn')}</button>` : `<span style="color:var(--text-muted);font-size:12px">${t('isClosedStatus')}</span>`}
      </td>
    </tr>`;
  }).join('') || `<tr><td colspan="6" style="text-align:center;padding:40px;color:var(--text-muted)">${t('noIssuesFound')}</td></tr>`;
  tbody.querySelectorAll('[data-resolve-issue]').forEach(b => b.addEventListener('click', () => openResolve(b.dataset.resolveIssue)));
  const filtered = State.issues.filter(x => ($('#iStatus')?.value ? x.status === $('#iStatus').value : true) && ($('#iSeverity')?.value ? x.severity === $('#iSeverity').value : true));
  renderPagination('#issuesPager', State.issuesTotal || 0, State.page.issues, State.issuesTotalPages || 1, (p) => { State.page.issues = p; loadIssues(true); });
}

$('#iStatus')?.addEventListener('change', renderIssues);
$('#iSeverity')?.addEventListener('change', renderIssues);

function openResolve(id) {
  openModal(t('capaModalTitle'), `
    <div class="field"><label>${t('rootCauseLabel')}</label><textarea id="capRoot" placeholder="${t('rootCausePlaceholder')}"></textarea></div>
    <div class="field"><label>${t('correctiveActionLabel')}</label><textarea id="capAct" placeholder="${t('correctiveActionPlaceholder')}"></textarea></div>
  `, `<button class="btn btn-success" id="saveCapa">${t('saveAndCloseBtn')}</button>
      <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);
  $('#saveCapa').addEventListener('click', async () => {
    try {
      const root = $('#capRoot').value.trim();
      const act = $('#capAct').value.trim();
      if (!root) throw new Error(t('rootCauseRequired'));
      await api('/issues/' + id + '/resolve', { method: 'PATCH', body: JSON.stringify({ root_cause: root, corrective_action: act })});
      State.loaded.issues = false;
      closeModal(); await loadIssues(); await loadDashboard();
      toast(t('issueClosedToast'));
    } catch (e) { toast(e.message, 'error'); }
  });
}

// ============ Report ============
function populateReportAgents() {
  const sel = $('#reportAgent');
  if (!sel) return;
  sel.innerHTML = '<option value="">' + t('reportSelectPlaceholder') + '</option>' + State.agents.map(a => `<option value="${a.id}">${esc(a.name)} — ${esc(a.department)}</option>`).join('');
  if (!sel.dataset.bound) {
    sel.addEventListener('change', renderReport);
    sel.dataset.bound = '1';
  }
  if (State.agents.length && !sel.value) {
    sel.value = State.agents[0].id;
  }
  renderReport();
}

async function renderReport() {
  const id = $('#reportAgent')?.value;
  if (!id) { $('#reportBody').innerHTML = ''; return; }
  try {
    const r = await api('/reports/agent/' + id);
    const kpis = [
      { label: t('kpiAvgScoreText'), value: Number(r.average_score || 0).toFixed(1), cls: 'success' },
      { label: t('reportScoredCount'), value: r.scored_interactions, cls: 'primary' },
      { label: t('reportCriticalFail'), value: r.critical_failures, cls: r.critical_failures > 0 ? 'danger' : 'success' },
    ];
    $('#reportBody').innerHTML = `
      <div class="kpi-grid">
        ${kpis.map(k => `<div class="kpi"><div class="kpi-label">${k.label}</div><div class="kpi-value ${k.cls}">${k.value}</div></div>`).join('')}
      </div>
      <div class="card">
        <table class="data-table">
          <thead><tr><th>${t('dateCol')}</th><th>${t('scoreCol')}</th><th>${t('levelCol')}</th><th>${t('statusCol')}</th></tr></thead>
          <tbody>${((r.scores || []).map(s => `
            <tr>
              <td>${fmtDate(s.created_at)}</td>
              <td><b>${Number(s.overall_score).toFixed(1)}</b></td>
              <td>${scorePill(s.overall_score, s.critical_fail)}</td>
              <td>${s.critical_fail ? '<span class="pill pill-bad">بحرانی</span>' : '<span class="pill pill-good">عادی</span>'}</td>
            </tr>
          `).join('') || `<tr><td colspan="4" style="text-align:center;padding:40px;color:var(--text-muted)">${t('notYetScored')}</td></tr>`)}
        </table>
      </div>
    `;
  } catch (e) { toast(e.message, 'error'); }
}

// ============ Modal helpers ============
function openModal(title, body, footer = '') {
  $('#modalTitle').textContent = title;
  $('#modalBody').innerHTML = body;
  $('#modalFooter').innerHTML = footer;
  $('#modal').classList.add('show');
  $$('#modalFooter [data-action="close-modal"]').forEach(b => b.addEventListener('click', closeModal));
}
function closeModal() { $('#modal').classList.remove('show'); }
document.addEventListener('click', e => {
  if (e.target.id === 'modal') closeModal();
  if (e.target.dataset?.action === 'new-interaction') openNewInteraction();
  if (e.target.dataset?.action === 'new-agent') openNewAgent();
  if (e.target.dataset?.action === 'new-customer') openNewCustomer();
  if (e.target.dataset?.action === 'close-modal') closeModal();
});

// ============ Users ============

let StateUsers = [];

async function loadUsers() {
  try {
    StateUsers = await api('/users');
    renderUsers();
  } catch (e) {
    if (e.message.includes('مدیر سیستم')) {
      $('#page-users').innerHTML = '<div class="card" style="text-align:center;padding:40px;color:var(--text-muted)">' + t('adminAccessOnly') + '</div>';
    } else { toast(e.message, 'error'); }
  }
}

function renderUsers() {
  const tbody = $('#usersTable tbody');
  if (!tbody) return;
  tbody.innerHTML = StateUsers.map(u => `
    <tr>
      <td><b>${esc(u.username)}</b></td>
      <td>${u.is_admin ? '<span class="pill pill-info">' + t('adminLabel') + '</span>' : '<span class="pill pill-muted">' + t('normalLabel') + '</span>'}</td>
      <td>${fmtDate(u.created_at)}</td>
      <td class="row-actions" data-label='${t("colActions")}'>
        <button class="btn btn-sm" data-edit-user="${u.username}">${t('editUserRoleBtn')}</button>
        <button class="btn btn-sm" data-del-user="${u.username}">${t('deleteCustLabel')}</button>
      </td>
    </tr>
  `).join('') || `<tr><td colspan="4" style="text-align:center;padding:40px;color:var(--text-muted)">${t('noUsersFound')}</td></tr>`;
  tbody.querySelectorAll('[data-edit-user]').forEach(b => b.addEventListener('click', () => openEditUser(b.dataset.editUser)));
  tbody.querySelectorAll('[data-del-user]').forEach(b => b.addEventListener('click', async () => {
    const u = b.dataset.delUser;
    if (u === State.user?.username) { toast(t('cannotDeleteSelf'), 'error'); return; }
    if (!confirm(t('confirmDeleteUser')(u))) return;
    try {
      await api('/users/' + encodeURIComponent(u), { method: 'DELETE' });
      await loadUsers();
      toast(t('userDeletedToast'));
    } catch (e) { toast(e.message, 'error'); }
  }));
}

$('#newUserBtn')?.addEventListener('click', () => openNewUser());

function openNewUser() {
  openModal(t('newUserTitle'), `
    <div class="field"><label>${t('usernameLabel')}</label><input id="uName" autocomplete="off"></div>
    <div class="field"><label>${t('passwordLabel')}</label><input id="uPass" type="password" autocomplete="new-password"></div>
    <div class="field"><label><input type="checkbox" id="uAdmin"> ${t('adminAccessLabel')}</label></div>
  `, `<button class="btn btn-primary" id="saveUser">${t('createBtn')}</button>
      <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);
  $('#saveUser').addEventListener('click', async () => {
    try {
      const username = $('#uName').value.trim();
      const password = $('#uPass').value;
      if (!username || !password) { toast(t('usernamePasswordRequired'), 'error'); return; }
      await api('/users', { method: 'POST', body: JSON.stringify({
        username, password, is_admin: $('#uAdmin').checked
      })});
      closeModal(); await loadUsers();
      toast(t('userCreatedToast'));
    } catch (e) { toast(e.message, 'error'); }
  });
}

function openEditUser(username) {
  const u = StateUsers.find(x => x.username === username);
  if (!u) return;
  openModal(t('editUserTitle')(username), `
    <div class="field"><label>نام کاربری</label><input value="${esc(username)}" disabled></div>
    <div class="field"><label>${t('newPasswordLabel')}</label><input id="uPassNew" type="password" autocomplete="new-password"></div>
    <div class="field"><label><input type="checkbox" id="uAdmin" ${u.is_admin ? 'checked' : ''}> ${t('adminAccessLabel')}</label></div>
  `, `<button class="btn btn-primary" id="updateUser">${t('saveBtn')}</button>
      <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);
  $('#updateUser').addEventListener('click', async () => {
    try {
      const newPass = $('#uPassNew').value;
      const body = { is_admin: $('#uAdmin').checked };
      if (newPass) body.password = newPass;
      await api('/users/' + encodeURIComponent(username), { method: 'PATCH', body: JSON.stringify(body) });
      closeModal(); await loadUsers();
      toast('کاربر بهروزرسانی شد');
    } catch (e) { toast(e.message, 'error'); }
  });
}

// ===================== Audit Log =====================
let auditPage = 1;
const auditPageSize = 50;

async function loadAudit(page = 1, force = false) {
  if (!force && State.loaded.audit && page === 1) { renderAudit(); return; }
  try {
    const action = $('#auditAction')?.value || '';
    const q = `?page=${page}&limit=${auditPageSize}` + (action ? `&action=${encodeURIComponent(action)}` : '');
    const data = await api('/audit/logs' + q);
    State.auditItems = data.items;
    State.auditTotal = data.total;
    State.auditTotalPages = data.total_pages;
    State.loaded.audit = true;
    renderAudit();
  } catch (e) {
    if (e.message.includes('دسترسی') || e.status === 403) {
      $('#page-audit').innerHTML = '<div class="card" style="text-align:center;padding:40px;color:var(--text-muted)">' + t('adminAccessOnly') + '</div>';
    } else {
      toast(e.message, 'error');
    }
  }
}

function renderAudit() {
  const tbody = $('#auditTable tbody');
  if (!tbody) return;
  const items = State.auditItems || [];
  tbody.innerHTML = items.map(a => `
    <tr>
      <td>${new Date(a.created_at).toLocaleString('fa-IR')}</td>
      <td>${a.username}</td>
      <td><span class="pill pill-info">${a.action_label || a.action}</span></td>
      <td>${a.resource_type || '-'}</td>
      <td title="${JSON.stringify(a.details || {}).toString()}">${a.summary || '-'}</td>
    </tr>
  `).join('');
  // pager
  const pagerEl = $('#auditPager');
  if (pagerEl) {
    const total = State.auditTotal || 0;
    const totalPages = State.auditTotalPages || 1;
    pagerEl.innerHTML = total > auditPageSize ?
      `<div class="pager" style="margin-bottom:12px"><button onclick="loadAudit(${Math.max(1,auditPage-1)})" ${auditPage<=1?'disabled':''}>${t('auditPrev')}</button>
       <span style="padding:0 12px">${t('auditPageInfo')(auditPage, totalPages, total)}</span>
       <button onclick="loadAudit(${auditPage+1})" ${auditPage>=totalPages?'disabled':''}>${t('auditNext')}</button></div>` :
      `<div style="margin-bottom:12px;color:var(--text-muted)">${t('auditTotalItems')(total)}</div>`;
  }
}

// ============ Boot ============
if (State.token) {
  enterApp();
}

// Health check / Redis status (polling every 10s)
async function checkConnection() {
  const el = $('#connStatus');
  if (!el) return;
  if (!el.querySelector('.dot')) {
    el.innerHTML = '<span class="dot"></span><span>...</span>';
  }
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 3000);
  try {
    const r = await fetch('/api/health?ts=' + Date.now(), { cache: 'no-store', signal: controller.signal });
    clearTimeout(timer);
    const j = await r.json();
    if (j && j.success) {
      el.classList.remove('disconnected');
      const txt = el.querySelector('span:last-child');
      if (txt) txt.textContent = t('connected');
    } else {
      throw new Error('invalid');
    }
  } catch (e) {
    const txt = el.querySelector('span:last-child');
    el.classList.add('disconnected');
    if (txt) txt.textContent = t('disconnected');
  }
}
checkConnection();
setInterval(checkConnection, 10000);


// ===================== Coaching Plans (Closed-Loop QA) =====================
const COACHING_STATUS_PILL = {
  'draft': 'pill-muted', 'pending_acknowledgement': 'pill-warn',
  'acknowledged': 'pill-info', 'in_progress': 'pill-info', 'verified': 'pill-good',
  'closed': 'pill-good', 'escalated': 'pill-bad'
};

function coachingStatusLabel(status) {
  return t('coaStatusMap')[status] || status;
}

let coachingPage = 1;
const coachingPageSize = 10;

function agentNameFor(agentId) {
  if (!agentId) return '-';
  const ag = (State.agents || []).find(a => String(a.id) === String(agentId));
  return ag ? (ag.name || agentId) : agentId;
}

async function loadCoaching(force = false) {
  const status = $('#cStatus')?.value || '';
  if (!force && State.loaded.coaching) { renderCoaching(); return; }
  try {
    const params = new URLSearchParams({ offset: (coachingPage - 1) * coachingPageSize, limit: coachingPageSize });
    if (status) params.set(t('status'), status);
    const data = await withLoading(t('coachLoadLabel'), () => api('/coaching/plans?' + params));
    State.coachingPlans = data.items || [];
    State.coachingTotal = data.total || State.coachingPlans.length;
    State.coachingTotalPages = Math.max(1, Math.ceil((data.total || 0) / coachingPageSize));
    State.loaded.coaching = true;
    setupCoachingEvents();
    renderCoaching();
  } catch (e) {
    toast(t('coachLoadedToast') + e.message, 'error');
  }
}

function setupCoachingEvents() {
  const statusSel = $('#cStatus');
  if (statusSel && !statusSel.dataset.bound) {
    statusSel.addEventListener('change', () => { coachingPage = 1; loadCoaching(true); });
    statusSel.dataset.bound = '1';
  }
  const newBtn = $('#newCoachingBtn');
  if (newBtn && !newBtn.dataset.bound) {
    newBtn.addEventListener('click', openNewCoachingPlan);
    newBtn.dataset.bound = '1';
  }
}

async function openNewCoachingPlan() {
  let agents = [];
  let interactions = [];
  try {
    await loadAgents();
    agents = State.agents || [];
  } catch (e) { /* ignore */ }
  try {
    const d = await api('/interactions?page=1&limit=1000');
    interactions = (d && d.items) || [];
  } catch (e) { /* ignore */ }
  const agentOpts = agents.length
    ? agents.map(a => `<option value="${esc(a.id)}">${esc(a.name)} — ${esc(a.department || '')}</option>`).join('')
    : '<option value="">— کارشناسی نیست —</option>';
  const intOpts = interactions.length
    ? interactions.map(i => `<option value="${esc(i.id)}">${esc(i.id)} — ${esc(i.subject)}</option>`).join('')
    : '<option value="">— تعاملی نیست —</option>';

  openModal(t('newCoachingPlanTitle'), `
    <div class="field"><label>${t('colAgent')}</label><select id="cpAgent">${agentOpts}</select></div>
    <div class="field"><label>${t('relatedInteractionLabel')}</label><select id="cpInteraction">${intOpts}</select></div>
    <div class="field"><label>${t('coachingThemeLabel')}</label><input id="cpTheme" placeholder="${t('coachingThemePlaceholder')}"></div>
    <div class="field"><label>${t('behaviorGapLabel')}</label><input id="cpGap" placeholder="${t('behaviorGapPlaceholder')}"></div>
    <div class="field"><label>${t('evidenceLabel')}</label><textarea id="cpEvidence" placeholder="${t('evidencePlaceholder')}"></textarea></div>
    <div class="field"><label>${t('rootCauseLabel')}</label><input id="cpRoot" placeholder="${t('rootCausePlaceholder')}"></div>
    <div class="field"><label>${t('customerImpactLabel')}</label><input id="cpImpact" placeholder="${t('customerImpactPlaceholder')}"></div>
    <div class="field"><label>${t('practiceActivityLabel')}</label><input id="cpActivity" placeholder="${t('practiceActivityPlaceholder')}"></div>
    <div class="field"><label>${t('successMetricLabel')}</label><input id="cpMetric" placeholder="${t('successMetricPlaceholder')}"></div>
    <div class="field"><label>${t('followUpCountLabel')}</label><input id="cpFollowups" type="number" min="1" value="2"></div>
    <div class="field"><label>${t('followUpDueLabel')}</label><input id="cpDue" type="datetime-local"></div>
  `, `<button class="btn btn-success" id="saveCoachingPlan">${t('createPlanBtn')}</button>
      <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);

  $('#saveCoachingPlan').addEventListener('click', async () => {
    try {
      const agentId = $('#cpAgent').value;
      const interactionId = $('#cpInteraction').value;
      if (!agentId) throw new Error(t('requiredAgentError'));
      if (!interactionId) throw new Error(t('interactionRequired'));
      const due = $('#cpDue').value;
      if (!due) throw new Error(t('dueDateRequired'));
      const body = {
        agent_id: agentId,
        interaction_id: interactionId,
        coaching_theme: $('#cpTheme').value.trim(),
        behavior_gap: $('#cpGap').value.trim(),
        evidence: $('#cpEvidence').value.trim(),
        root_cause: $('#cpRoot').value.trim(),
        customer_impact: $('#cpImpact').value.trim(),
        practice_activity: $('#cpActivity').value.trim(),
        success_metric: $('#cpMetric').value.trim(),
        follow_up_due_at: new Date(due).toISOString(),
        follow_up_review_count: parseInt($('#cpFollowups').value, 10) || 2
      };
      await withLoading(t('createCoachLabel'), () => api('/coaching/plans', { method: 'POST', body: JSON.stringify(body) }));
      closeModal();
      State.loaded.coaching = false;
      await loadCoaching(true);
      toast(t('coachingCreatedToast'), 'success');
    } catch (e) { toast(e.message, 'error'); }
  });
}

function renderCoaching() {
  const tbody = document.querySelector('#coachingTable tbody');
  if (!tbody) return;
  const plans = State.coachingPlans || [];
  if (plans.length === 0) {
    tbody.innerHTML = '<tr><td colspan="9" style="text-align:center;padding:40px;color:var(--text-muted)">' + t('noCoachingPlans') + '</td></tr>';
    renderPagination('#coachingPager', 0, 1, 1, () => {});
    return;
  }
  tbody.innerHTML = plans.map(p => {
    const pill = COACHING_STATUS_PILL[p.status] || 'pill-muted';
    return `<tr>
      <td><span class="pill ${pill}">${esc(coachingStatusLabel(p.status))}</span></td>
      <td>${esc(agentNameFor(p.agent_id))}</td>
            <td title="${esc(p.coaching_theme)}">${esc(p.coaching_theme)}</td>
      <td title="${esc(p.behavior_gap)}">${esc(p.behavior_gap || '-')}</td>
      <td title="${esc(p.root_cause)}">${esc(p.root_cause || '-')}</td>
      <td title="${esc(p.customer_impact)}">${esc(p.customer_impact || '-')}</td>
      <td>${esc(p.success_metric || '-')}</td>
      <td>${fmtDate(p.follow_up_due_at)}</td>
      <td class="row-actions" data-label='${t("colActions")}'>${renderCoachingActions(p)}</td>
    </tr>`;
  }).join('');

  // Attach action handlers
  tbody.querySelectorAll('[data-action-coaching]').forEach(btn => {
    btn.addEventListener('click', (e) => {
      const action = e.currentTarget.dataset.actionCoaching;
      const planId = e.currentTarget.dataset.planId;
      handleCoachingAction(action, planId);
    });
  });

  renderPagination('#coachingPager', State.coachingTotal || 0, coachingPage, State.coachingTotalPages || 1, (p) => {
    coachingPage = p;
    loadCoaching(true);
  });
}

function renderCoachingActions(p) {
  const actions = [];
  if (p.status === 'draft') {
    actions.push(`<button class="btn btn-sm btn-primary" data-action-coaching="submit" data-plan-id="${esc(p.id)}">${t('submitBtn')}</button>`);
  } else if (p.status === 'pending_acknowledgement') {
    actions.push(`<button class="btn btn-sm btn-success" data-action-coaching="acknowledge" data-plan-id="${esc(p.id)}">${t('approveBtn')}</button>`);
  } else if (p.status === 'acknowledged' || p.status === 'in_progress') {
    actions.push(`<button class="btn btn-sm btn-secondary" data-action-coaching="review" data-plan-id="${esc(p.id)}">${t('detailsBtn')}</button>`);
    actions.push(`<button class="btn btn-sm btn-danger" data-action-coaching="close" data-plan-id="${esc(p.id)}">${t('closeBtn2')}</button>`);
  } else if (p.status === 'verified') {
    actions.push(`<button class="btn btn-sm btn-secondary" data-action-coaching="review" data-plan-id="${esc(p.id)}">${t('detailsBtn')}</button>`);
    actions.push(`<button class="btn btn-sm btn-danger" data-action-coaching="close" data-plan-id="${esc(p.id)}">${t('closeBtn2')}</button>`);
  } else if (p.status === 'escalated') {
    actions.push(`<button class="btn btn-sm btn-secondary" data-action-coaching="review" data-plan-id="${esc(p.id)}">${t('reviewDetailsBtn')}</button>`);
    actions.push(`<button class="btn btn-sm btn-warning" data-action-coaching="add-note" data-plan-id="${esc(p.id)}">${t('addNoteBtn')}</button>`);
    actions.push(`<button class="btn btn-sm btn-primary" data-action-coaching="resume" data-plan-id="${esc(p.id)}">${t('resumeBtn')}</button>`);
  }
  if (['pending_acknowledgement','acknowledged','in_progress'].includes(p.status)) {
    actions.push(`<button class="btn btn-sm btn-ghost" data-action-coaching="escalate" data-plan-id="${esc(p.id)}">${t('escalateBtn')}</button>`);
  }
  return actions.join(' ');
}

async function handleCoachingAction(action, planId) {
  try {
    if (action === 'review') {
      await openCoachingReview(planId);
      return;
    }
    if (action === 'acknowledge') {
      await api('/coaching/plans/' + planId + '/acknowledge', { method: 'POST', body: JSON.stringify({ note: t('planApproved') }) });
      toast(t('planApprovedToast'), 'success');
    } else if (action === 'close') {
      const outcome = prompt('نتیجه بسته شدن (مثلاً improved / unchanged):', 'improved');
      if (outcome === null) return;
      await api('/coaching/plans/' + planId + '/close', { method: 'POST', body: JSON.stringify({ outcome: outcome || 'improved' }) });
      toast(t('planClosedToast'), 'success');
    } else if (action === 'escalate') {
      if (!confirm(t('confirmEscalate'))) return;
      await api('/coaching/plans/' + planId + '/escalate', { method: 'POST' });
      toast(t('planEscalatedToast'), 'warning');
    } else if (action === 'resume') {
      if (!confirm(t('confirmResume'))) return;
      await api('/coaching/plans/' + planId + '/resume', { method: 'POST' });
      toast(t('planResumedToast'), 'success');
    } else if (action === 'add-note') {
      const note = prompt('یادداشت مدیر (دلیل ارجاع یا تصمیم):', '');
      if (note === null) return;
      if (!note.trim()) { toast('یادداشت نمی‌تواند خالی باشد', 'error'); return; }
      await api('/coaching/plans/' + planId + '/acknowledge', { method: 'POST', body: JSON.stringify({ note: note.trim() }) });
      toast(t('noteSavedToast'), 'success');
    } else if (action === 'submit') {
      await api('/coaching/plans/' + planId + '/submit', { method: 'POST' });
      toast(t('planSubmittedToast'), 'success');
    }
    State.loaded.coaching = false;
    await loadCoaching(true);
  } catch (e) {
    toast(e.message, 'error');
  }
}

async function openCoachingReview(planId) {
  showLoading(t('loadingAllLabel'));
  try {
    const data = await api('/coaching/plans/' + planId);
    hideLoading();
    const plan = data.plan;
    const fups = data.follow_ups || [];
    const labels = COACHING_STATUS_PILL; // Use pill map only, labels via function
    const followUpTable = fups.length
      ? `<table class="data-table"><thead><tr><th>${t('colDate')}</th><th>${t('scoreCol')} کلی</th><th>موفق</th></tr></thead><tbody>
           ${fups.map(f => `<tr><td>${fmtDate(f.measured_at)}</td><td><b>${Number(f.overall_score).toFixed(1)}</b></td><td>${f.success ? '✅' : '❌'}</td></tr>`).join('')}
         </tbody></table>`
      : '<p style="color:var(--text-muted)">' + t('noFollowUpsYet') + '</p>';
    openModal(t('coachingReviewTitle'), `
      <div class="field"><b>موضوع آموزشی:</b> ${esc(plan.coaching_theme)}</div>
      <div class="field"><b>${t('behaviorGapLabel')}:</b> ${esc(plan.behavior_gap)}</div>
      <div class="field"><b>شواهد:</b> ${esc(plan.evidence)}</div>
      <div class="field"><b>علت ریشه:</b> ${esc(plan.root_cause)}</div>
      <div class="field"><b>اثر بر مشتری:</b> ${esc(plan.customer_impact)}</div>
      <div class="field"><b>فعالیت عملی:</b> ${esc(plan.practice_activity)}</div>
      <div class="field"><b>سنجه موفقیت:</b> ${esc(plan.success_metric)}</div>
      <div class="field"><b>${t('statusLabel')}:</b> <span class="pill ${labels[plan.status] || 'pill-muted'}">${esc(coachingStatusLabel(plan.status))}</span></div>
      <h4 style="margin:16px 0 8px">${t('followUpsLabel')} (${fups.length})</h4>
      ${followUpTable}
    `, `<button class="btn" data-action="close-modal">${t('closeBtn2')}</button>`);
  } catch (e) {
    hideLoading();
    toast(e.message, 'error');
  }
}

// ===================== Calibration Sessions (Blind Scoring) =====================
const CAL_STATUS_PILL = {
  'draft': 'pill-muted', 'scoring': 'pill-warn', 'in_session': 'pill-info',
  'completed': 'pill-good', 'cancelled': 'pill-bad'
};

function calStatusLabel(status) {
  return t('calStatusMap')[status] || status;
}

let calPage = 1;
const calPageSize = 10;

async function loadCalibration(force = false) {
  const status = $('#calStatus')?.value || '';
  if (!force && State.loaded.calibration) { renderCalibration(); return; }
  try {
    const params = new URLSearchParams({ offset: (calPage - 1) * calPageSize, limit: calPageSize });
    if (status) params.set(t('status'), status);
    const data = await withLoading(t('calLoadLabel'), () => api('/calibration/sessions?' + params));
    State.calibrationSessions = data.items || [];
    State.calibrationTotal = data.total || State.calibrationSessions.length;
    State.calibrationTotalPages = Math.max(1, Math.ceil((data.total || 0) / calPageSize));
    State.loaded.calibration = true;
    attachCalibrationEventListeners();
    renderCalibration();
  } catch (e) {
    toast(t('calLoadedToast') + e.message, 'error');
  }
}

function attachCalibrationEventListeners() {
  const btn = $('#newCalBtn');
  if (btn && !btn.dataset.bound) {
    btn.addEventListener('click', openNewCalibration);
    btn.dataset.bound = '1';
  }
  const sel = $('#calStatus');
  if (sel && !sel.dataset.bound) {
    sel.addEventListener('change', () => { calPage = 1; loadCalibration(true); });
    sel.dataset.bound = '1';
  }
}

function renderCalibration() {
  const tbody = document.querySelector('#calTable tbody');
  if (!tbody) return;
  const sessions = State.calibrationSessions || [];
  if (sessions.length === 0) {
    tbody.innerHTML = '<tr><td colspan="7" style="text-align:center;padding:40px;color:var(--text-muted)">' + t('noCalibrationSessions') + '</td></tr>';
    renderPagination('#calPager', 0, 1, 1, () => {});
    return;
  }
  tbody.innerHTML = sessions.map(s => {
    const pill = CAL_STATUS_PILL[s.status] || 'pill-muted';
    const rate = s.agreement_rate != null ? (s.agreement_rate * 100).toFixed(1) + '%' : '—';
    return `<tr>
      <td data-label='${t("statusCol")}'><span class="pill ${pill}">${esc(calStatusLabel(s.status))}</span></td>
      <td data-label='${t("colSessionName")}'>${esc(s.name)}</td>
      <td data-label='${t("colStandard")}'>${esc(s.rubric_id || '-')}</td>
      <td data-label='${t("colSamples")}'>${(s.sample_interaction_ids || []).length}</td>
      <td data-label='${t("colAgreement")}'><b>${rate}</b></td>
      <td data-label='${t("colDeadline")}'>${fmtDate(s.deadline_at)}</td>
      <td class="row-actions" data-label='${t("colActions")}'>${renderCalibrationActions(s)}</td>
    </tr>`;
  }).join('');

  tbody.querySelectorAll('[data-action-cal]').forEach(btn => {
    btn.addEventListener('click', (e) => {
      const action = e.currentTarget.dataset.actionCal;
      const id = e.currentTarget.dataset.sessionId;
      handleCalibrationAction(action, id);
    });
  });

  renderPagination('#calPager', State.calibrationTotal || 0, calPage, State.calibrationTotalPages || 1, (p) => {
    calPage = p;
    loadCalibration(true);
  });
}

function renderCalibrationActions(s) {
  const a = [];
  if (s.status === 'draft') {
    a.push(`<button class="btn btn-sm btn-primary" data-action-cal="start" data-session-id="${esc(s.id)}">شروع</button>`);
  } else if (s.status === 'scoring') {
    a.push(`<button class="btn btn-sm btn-secondary" data-action-cal="score" data-session-id="${esc(s.id)}">t('scoreCol')دهی</button>`);
    a.push(`<button class="btn btn-sm btn-success" data-action-cal="meeting" data-session-id="${esc(s.id)}">شروع جلسه</button>`);
  } else if (s.status === 'in_session') {
    a.push(`<button class="btn btn-sm btn-primary" data-action-cal="complete" data-session-id="${esc(s.id)}">تکمیل</button>`);
  }
  if (['draft','scoring'].includes(s.status)) {
    a.push(`<button class="btn btn-sm btn-ghost" data-action-cal="cancel" data-session-id="${esc(s.id)}">t('cancelBtn3')</button>`);
  }
  return a.join(' ');
}

async function handleCalibrationAction(action, id) {
  try {
    if (action === 'start') {
      await api('/calibration/sessions/' + id + '/transition', { method: 'POST', body: JSON.stringify({ to_status: 'start' }) });
      toast(`جلسه شروع شد (در حال ${t('scoreCol')})دهی)`, 'success');
    } else if (action === 'meeting') {
      await api('/calibration/sessions/' + id + '/transition', { method: 'POST', body: JSON.stringify({ to_status: 'begin-meeting' }) });
      toast('جلسه حضوری شروع شد', 'success');
    } else if (action === 'complete') {
      await api('/calibration/sessions/' + id + '/transition', { method: 'POST', body: JSON.stringify({ to_status: 'complete' }) });
      toast(`جلسه تکمیل و نرخ ${t('agreementCol')} محاسبه شد`, 'success');
    } else if (action === 'cancel') {
      if (!confirm(`جلسه ${t('cancelBtn3')} شود؟`)) return;
      await api('/calibration/sessions/' + id + '/transition', { method: 'POST', body: JSON.stringify({ to_status: 'cancel' }) });
      toast(`جلسه ${t('cancelBtn3')} شد`, 'warning');
    } else if (action === 'score') {
      await openCalibrationScoring(id);
      return;
    }
    State.loaded.calibration = false;
    await loadCalibration(true);
  } catch (e) {
    toast(e.message, 'error');
  }
}

async function openNewCalibration() {
  // Load rubrics + agents (for reviewers) to populate the form
  let rubrics = [];
  let agents = [];
  let users = [];
  try {
    const r = await api('/rubrics');
    rubrics = r || [];
  } catch (e) { /* ignore */ }
  try {
    const a = await api('/agents?page=1&limit=1000');
    agents = (a && a.items) || [];
  } catch (e) { /* ignore */ }
  const rubricOptions = rubrics.length
    ? rubrics.map(x => `<option value="${esc(x.id)}">${esc(x.name || x.id)}</option>`).join('')
    : '<option value="">— بدون روبریک —</option>';
  const reviewerOptions = agents.length
    ? agents.map(a => `<option value="${esc(a.id)}">${esc(a.name)}</option>`).join('')
    : '';
  const interactionOptions = (State.interactions || []).map(i => `<option value="${esc(i.id)}">${esc(i.id)} — ${esc(i.subject)}</option>`).join('');

  openModal(t('newCalibrationSessionTitle'), `
    <div class="field"><label>${t('sessionNameLabel')}</label><input id="calName" placeholder="${t('calSessionPlaceholder')}"></div>
    <div class="field"><label>${t('rubricLabel')}</label><select id="calRubric">${rubricOptions || '<option value="">' + t('noRubricsOption') + '</option>'}</select></div>
    <div class="field"><label>${t('endDateLabel')}</label><input id="calDeadline" type="datetime-local"></div>
    <div class="field"><label>${t('targetAgreementLabel')}</label><input id="calTarget" type="number" step="0.05" min="0" max="1" value="0.8"></div>
    <div class="field"><label>${t('reviewersLabel')}</label>
      <select id="calReviewers" multiple size="4">${reviewerOptions || '<option value="">' + t('noAgentsOption') + '</option>'}</select></div>
    <div class="field"><label>${t('sampleInteractionsLabel')}</label>
      <select id="calSamples" multiple size="4">${interactionOptions || '<option value="">' + t('noInteractionsOption') + '</option>'}</select></div>
  `, `<button class="btn btn-success" id="saveCal">${t('createSessionBtn')}</button>
      <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);

  $('#saveCal').addEventListener('click', async () => {
    try {
      const name = $('#calName').value.trim();
      if (!name) throw new Error(t('sessionNameRequired'));
      const reviewer_usernames = Array.from($('#calReviewers').selectedOptions).map(o => o.value);
      const sample_interaction_ids = Array.from($('#calSamples').selectedOptions).map(o => o.value);
      const dt = $('#calDeadline').value;
      if (!dt) throw new Error(t('deadlineRequired'));
      const body = {
        name,
        rubric_id: $('#calRubric').value,
        reviewer_usernames,
        sample_interaction_ids,
        target_agreement_rate: parseFloat($('#calTarget').value) || 0.8,
        min_reviewers_per_interaction: 2,
        deadline_at: new Date(dt).toISOString()
      };
      await withLoading(t('createCalLabel'), () => api('/calibration/sessions', { method: 'POST', body: JSON.stringify(body) }));
      closeModal();
      State.loaded.calibration = false;
      await loadCalibration(true);
      toast(t('calibrationCreatedToast'), 'success');
    } catch (e) { toast(e.message, 'error'); }
  });
}

async function openCalibrationScoring(sessionId) {
  showLoading(t('calSessionLoadLabel'));
  try {
    const s = await api('/calibration/sessions/' + sessionId);
    hideLoading();
    const samples = s.sample_interaction_ids || [];
    if (samples.length === 0) { toast(t('noSampleInteractions'), 'warning'); return; }
    const sampleRows = samples.map(it => `
      <tr>
        <td>${esc(it)}</td>
        <td><input type="number" class="calScore" data-interaction="${esc(it)}" min="0" max="100" value="50" style="width:90px"></td>
        <td><input type="text" class="calNotes" data-interaction="${esc(it)}" placeholder="${t('notesOptionalPlaceholder')}" style="width:100%"></td>
      </tr>`).join('');
    openModal(t('calibrationScoringTitle'), `
      <p style="color:var(--text-muted);margin-bottom:12px">t('scoreCol') هر تعامل را ۰ تا ۱۰۰ بگذارید. t('scoreCol')ها بهصورت کور (blind) ${t('registerBtn')}) میشوند.</p>
      <table class="data-table"><thead><tr><th>${t('interactionCol')}</th><th>${t('overallScoreCol')}</th><th>${t('notesCol')}</th></tr></thead><tbody>${sampleRows}</tbody></table>
    `, `<button class="btn btn-success" id="saveCalScore">${t('saveScoresBtn')}</button>
        <button class="btn" data-action="close-modal">${t('cancelBtn2')}</button>`);
    $('#saveCalScore').addEventListener('click', async () => {
      try {
        const scores = samples.map(it => {
          const val = parseFloat(document.querySelector(`.calScore[data-interaction="${it}"]`).value);
          if (isNaN(val)) throw new Error(t('invalidScore') + ' ' + it);
          return {
            interaction_id: it,
            criterion_scores: {},      // blank JSONB object; overall is the aggregate
            overall_score: val,
            notes: document.querySelector(`.calNotes[data-interaction="${it}"]`).value || null
          };
        });
        await withLoading(t('submitScoresLabel'), async () => {
          for (const sc of scores) {
            await api('/calibration/sessions/' + sessionId + '/score', { method: 'POST', body: JSON.stringify(sc) });
          }
        });
        closeModal();
        toast(t('scoresSavedToast'), 'success');
      } catch (e) { toast(e.message, 'error'); }
    });
  } catch (e) {
    hideLoading();
    toast(e.message, 'error');
  }
}

