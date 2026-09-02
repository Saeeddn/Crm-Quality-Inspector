// CRM Quality Inspector - frontend v0.4
// Features: KPI management, auto-scoring, predictive risk queue, manual override

const $ = (s) => document.querySelector(s);
const $$ = (s) => Array.from(document.querySelectorAll(s));

const TOKEN_KEY = 'crm_qi_token';
const USER_KEY = 'crm_qi_user';
const CACHE_KEY = 'crm_qi_cache_v1';
const CACHE_TTL = 60_000;

const State = {
  token: localStorage.getItem(TOKEN_KEY) || null,
  user: JSON.parse(localStorage.getItem(USER_KEY) || 'null'),
  agents: [], customers: [], interactions: [], rubrics: [],
  scores: {}, issues: [], recommendations: [], kpis: [],
  dashboard: null, agentsAvg: {},
  loaded: { agents: false, customers: false, interactions: false, issues: false, rubrics: false, kpis: false, dashboard: false, rec: false },
  trendChart: null,
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
  const text = await r.text();
  if (!text) {
    if (r.ok) return null;
    throw new Error('سرور پاسخ خالی داد (کد ' + r.status + ')');
  }
  let j;
  try { j = JSON.parse(text); } catch { throw new Error('پاسخ نامعتبر'); }
  if (!j.success) throw new Error(j.error || 'خطا');
  return j.data;
}

function esc(s) {
  return String(s ?? '').replace(/[&<>"']/g, m => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[m]));
}
function fmtDate(iso) {
  if (!iso) return '-';
  try { return new Date(iso).toLocaleString('fa-IR', { dateStyle: 'short', timeStyle: 'short' }); }
  catch { return iso; }
}
function scorePill(v, critical) {
  if (v == null) return '<span class="pill pill-muted">ارزیابی‌نشده</span>';
  const cls = critical ? 'pill-bad' : v >= 85 ? 'pill-good' : v >= 70 ? 'pill-info' : v >= 60 ? 'pill-warn' : 'pill-bad';
  return `<span class="pill ${cls}">${Number(v).toFixed(1)}${critical ? ' ⚠' : ''}</span>`;
}
function sevPill(s) {
  const map = { 'بحرانی': 'pill-bad', 'بالا': 'pill-warn', 'متوسط': 'pill-info', 'پایین': 'pill-muted' };
  return `<span class="pill ${map[s] || 'pill-muted'}">${esc(s)}</span>`;
}
function statusPill(s) {
  return s === 'باز' ? '<span class="pill pill-warn">باز</span>' : '<span class="pill pill-good">بسته</span>';
}
function priorityPill(p) {
  const map = { 'بالا': 'pill-bad', 'متوسط': 'pill-warn', 'پایین': 'pill-info' };
  return `<span class="pill ${map[p] || 'pill-muted'}">اولویت ${esc(p)}</span>`;
}
function toast(msg, type = 'success') {
  const t = $('#toast');
  t.textContent = msg;
  t.className = 'toast show ' + type;
  setTimeout(() => t.classList.remove('show'), 3500);
}

// ============ Login ============
$('#loginForm').addEventListener('submit', async (e) => {
  e.preventDefault();
  const username = $('#loginUser').value.trim();
  const password = $('#loginPass').value;
  const err = $('#loginError');
  err.classList.remove('show');
  try {
    const data = await api('/auth/login', { method: 'POST', body: JSON.stringify({ username, password }) });
    setToken(data.token, { username: data.username, is_admin: data.is_admin });
    enterApp();
  } catch (ex) {
    err.textContent = ex.message;
    err.classList.add('show');
  }
});

function logout() {
  setToken(null, null);
  State.agents = []; State.customers = []; State.interactions = [];
  State.rubrics = []; State.scores = {}; State.issues = [];
  State.recommendations = []; State.kpis = []; State.dashboard = null;
  Object.keys(State.loaded).forEach(k => State.loaded[k] = false);
  $('#loginScreen').classList.remove('hidden');
  $('#appShell').classList.add('hidden');
}

// ============ App ============
async function enterApp() {
  $('#loginScreen').classList.add('hidden');
  $('#appShell').classList.remove('hidden');
  $('#userBadge').textContent = State.user?.username || '';
  await loadDashboard();
  switchTab('dashboard');
}

async function loadDashboard() {
  if (State.loaded.dashboard) { renderDashboard(); return; }
  const c = cacheGet('dashboard'); if (c) { State.dashboard = c; State.loaded.dashboard = true; renderDashboard(); return; }
  try {
    State.dashboard = await api('/reports/dashboard');
    State.loaded.dashboard = true;
    cacheSet('dashboard', State.dashboard);
    renderDashboard();
  } catch (e) {
    if (e.message.includes('invalid or expired') || e.message.includes('missing bearer')) logout();
    else toast(e.message, 'error');
  }
}

async function loadAgents() {
  if (State.loaded.agents) { renderAgents(); return; }
  const c = cacheGet('agents'); if (c) { State.agents = c; State.loaded.agents = true; renderAgents(); return; }
  State.agents = await api('/agents'); State.loaded.agents = true; cacheSet('agents', State.agents);
  renderAgents();
}
async function loadCustomers() {
  if (State.loaded.customers) { renderCustomers(); return; }
  const c = cacheGet('customers'); if (c) { State.customers = c; State.loaded.customers = true; renderCustomers(); return; }
  State.customers = await api('/customers'); State.loaded.customers = true; cacheSet('customers', State.customers);
  renderCustomers();
}
async function loadInteractions() {
  if (State.loaded.interactions) { renderInteractions(); return; }
  const c = cacheGet('interactions'); if (c) { State.interactions = c; State.loaded.interactions = true; renderInteractions(); return; }
  State.interactions = await api('/interactions');
  State.loaded.interactions = true;
  cacheSet('interactions', State.interactions);
  loadAllScoresLazy();
  renderInteractions();
}
async function loadAllScoresLazy() {
  const promises = State.interactions.map(it =>
    api('/scoring/' + it.id).then(s => { if (s) State.scores[it.id] = s; }).catch(() => null)
  );
  await Promise.all(promises);
  renderInteractions();
}
async function loadIssues() {
  if (State.loaded.issues) { renderIssues(); return; }
  State.issues = await api('/issues'); State.loaded.issues = true;
  renderIssues();
}
async function loadKpis() {
  if (State.loaded.kpis) { renderKpis(); return; }
  State.kpis = await api('/kpis'); State.loaded.kpis = true;
  renderKpis();
}
async function loadRecommendations() {
  if (State.loaded.rec) { renderRecommendations(); return; }
  State.recommendations = await api('/recommendations'); State.loaded.rec = true;
  renderRecommendations();
}

function switchTab(tab) {
  $$('.page').forEach(p => p.classList.add('hidden'));
  $$('.nav-item').forEach(n => n.classList.remove('active'));
  $('#page-' + tab).classList.remove('hidden');
  $(`.nav-item[data-tab="${tab}"]`)?.classList.add('active');
  $('#topbarTitle').textContent = {
    dashboard: 'داشبورد', interactions: 'تعاملات', agents: 'کارشناسان',
    customers: 'مشتریان', recommendations: 'پیشنهادهای QA', issues: 'ایرادات',
    rubrics: 'پارامترهای اندازه‌گیری', report: 'گزارش کارشناس',
  }[tab] || tab;
  if (tab === 'dashboard') loadDashboard();
  else if (tab === 'interactions') loadInteractions();
  else if (tab === 'agents') loadAgents();
  else if (tab === 'customers') loadCustomers();
  else if (tab === 'issues') loadIssues();
  else if (tab === 'rubrics') loadKpis();
  else if (tab === 'recommendations') loadRecommendations();
  else if (tab === 'report') loadAgents().then(populateReportAgents);
}

$$('.nav-item').forEach(n => n.addEventListener('click', () => switchTab(n.dataset.tab)));
$('#logoutBtn').addEventListener('click', logout);

// ============ Dashboard ============
function renderDashboard() {
  const d = State.dashboard || {};
  const kpis = [
    { label: 'کارشناسان', value: d.agent_count, cls: 'primary' },
    { label: 'مشتریان', value: d.customer_count, cls: 'info' },
    { label: 'تعاملات', value: d.interaction_count, cls: 'primary' },
    { label: 'پوشش ارزیابی', value: (d.coverage || 0).toFixed(1) + '%', cls: 'info' },
    { label: 'میانگین کیفیت', value: (d.average_score || 0).toFixed(1), cls: d.average_score >= 80 ? 'success' : d.average_score >= 60 ? 'warning' : 'danger' },
    { label: 'ایرادات باز', value: d.open_issues, cls: d.open_issues > 0 ? 'warning' : 'success' },
    { label: 'شکست بحرانی', value: d.critical_failures, cls: d.critical_failures > 0 ? 'danger' : 'success' },
    { label: 'گرید کیفیت', value: d.quality_grade, cls: 'primary' },
  ];
  $('#kpiGrid').innerHTML = kpis.map(k =>
    `<div class="kpi"><div class="kpi-label">${k.label}</div><div class="kpi-value ${k.cls}">${k.value}</div></div>`
  ).join('');
  $('#coverageBar').innerHTML = `
    <div style="display:flex;justify-content:space-between;margin-bottom:8px">
      <span style="color:var(--text-muted)">${d.scored_count || 0} از ${d.interaction_count || 0} تعامل ارزیابی شده</span>
      <strong>${(d.coverage || 0).toFixed(1)}%</strong>
    </div>
    <div class="bar-track"><div class="bar-fill" style="width:${Math.min(100, d.coverage || 0)}%"></div></div>
  `;
  renderTrendChart();
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
    ctx.fillText('برای نمایش نمودار، حداقل ۲ ارزیابی لازم است', 10, 30);
    return;
  }
  const sorted = scores.slice().sort((a, b) => new Date(a.created_at) - new Date(b.created_at));
  const labels = sorted.map(s => fmtDate(s.created_at));
  const data = sorted.map(s => s.overall_score);
  State.trendChart = new Chart(canvas, {
    type: 'line',
    data: { labels, datasets: [{
      label: 'میانگین امتیاز کیفیت',
      data,
      borderColor: '#6366f1',
      backgroundColor: 'rgba(99,102,241,0.15)',
      fill: true, tension: 0.3, pointRadius: 4, pointBackgroundColor: '#8b5cf6',
    }] },
    options: {
      responsive: true, maintainAspectRatio: false,
      plugins: { legend: { labels: { color: '#e6e8ee' } } },
      scales: {
        x: { ticks: { color: '#8a92a6' }, grid: { color: '#2a2f3d' } },
        y: { min: 0, max: 100, ticks: { color: '#8a92a6' }, grid: { color: '#2a2f3d' } },
      },
    },
  });
}

// ============ Interactions ============
function renderInteractions() {
  const tbody = $('#interactionsTable tbody');
  if (!tbody) return;
  const search = ($('#fSearch')?.value || '').toLowerCase();
  const channel = $('#fChannel')?.value || '';
  const agentId = $('#fAgent')?.value || '';
  const status = $('#fStatus')?.value || '';

  const sel = $('#fAgent');
  if (sel && sel.options.length <= 1 && State.agents.length) {
    sel.innerHTML = '<option value="">همه کارشناسان</option>' + State.agents
      .filter(a => a.active).map(a => `<option value="${a.id}">${esc(a.name)}</option>`).join('');
  }

  let rows = State.interactions.slice();
  if (search) rows = rows.filter(i => (i.subject + ' ' + i.transcript + ' ' + (i.tags||[]).join(' ')).toLowerCase().includes(search));
  if (channel) rows = rows.filter(i => i.channel === channel);
  if (agentId) rows = rows.filter(i => i.agent_id === agentId);
  if (status === 'scored') rows = rows.filter(i => State.scores[i.id]);
  if (status === 'unscored') rows = rows.filter(i => !State.scores[i.id]);
  rows.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));

  tbody.innerHTML = rows.map(i => {
    const s = State.scores[i.id];
    const agent = State.agents.find(a => a.id === i.agent_id);
    const customer = State.customers.find(c => c.id === i.customer_id);
    return `<tr>
      <td>${fmtDate(i.created_at)}</td>
      <td>${esc(agent?.name || '-')}</td>
      <td>${esc(customer?.name || '-')}</td>
      <td><span class="pill pill-muted">${esc(i.channel)}</span></td>
      <td><b>${esc(i.subject)}</b><div style="color:var(--text-muted);font-size:12px;margin-top:2px">${esc((i.transcript || '').slice(0, 80))}${(i.transcript || '').length > 80 ? '…' : ''}</div></td>
      <td>${scorePill(s?.overall_score, s?.critical_fail)}</td>
      <td class="row-actions">
        <button class="btn btn-sm btn-primary" data-auto="${i.id}">${s ? 'بازبینی' : 'ارزیابی خودکار'}</button>
        <button class="btn btn-sm" data-view="${i.id}">مشاهده</button>
      </td>
    </tr>`;
  }).join('') || `<tr><td colspan="7" style="text-align:center;padding:40px;color:var(--text-muted)">تعاملی یافت نشد</td></tr>`;

  tbody.querySelectorAll('[data-auto]').forEach(b => b.addEventListener('click', () => openAutoScore(b.dataset.auto)));
  tbody.querySelectorAll('[data-view]').forEach(b => b.addEventListener('click', () => openView(b.dataset.view)));
}

$('#fSearch')?.addEventListener('input', renderInteractions);
$('#fChannel')?.addEventListener('change', renderInteractions);
$('#fAgent')?.addEventListener('change', renderInteractions);
$('#fStatus')?.addEventListener('change', renderInteractions);
$('#exportCsvBtn')?.addEventListener('click', exportCsv);

function exportCsv() {
  if (!State.interactions.length) { toast('ابتدا تعاملات را بارگذاری کنید', 'error'); return; }
  const rows = [['شناسه', 'تاریخ', 'کارشناس', 'مشتری', 'کانال', 'موضوع', 'امتیاز', 'سطح', 'بحرانی', 'یادداشت']];
  for (const i of State.interactions) {
    const s = State.scores[i.id];
    const agent = State.agents.find(a => a.id === i.agent_id);
    const customer = State.customers.find(c => c.id === i.customer_id);
    rows.push([i.id, fmtDate(i.created_at), agent?.name || '', customer?.name || '',
      i.channel, i.subject, s ? s.overall_score : '', s ? s.level : '',
      s ? (s.critical_fail ? 'بله' : 'خیر') : '', s?.notes || '']);
  }
  const csv = '\uFEFF' + rows.map(r => r.map(c => `"${String(c).replace(/"/g, '""')}"`).join(',')).join('\n');
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url; a.download = `interactions-${new Date().toISOString().slice(0,10)}.csv`;
  a.click(); URL.revokeObjectURL(url);
  toast('فایل CSV دانلود شد');
}

// ============ Auto-Score Modal ============
async function openAutoScore(id) {
  const interaction = State.interactions.find(i => i.id === id);
  if (!interaction) return;
  if (State.kpis.length === 0) await loadKpis();
  const customer = State.customers.find(c => c.id === interaction.customer_id);
  const agent = State.agents.find(a => a.id === interaction.agent_id);

  // First show a preview/measure
  openModal('ارزیابی خودکار', `
    <div style="background:var(--surface-2);padding:12px;border-radius:8px;margin-bottom:14px">
      <b>${esc(interaction.subject)}</b>
      <div style="color:var(--text-muted);font-size:12px;margin-top:4px">${esc(interaction.transcript.slice(0, 300))}${interaction.transcript.length > 300 ? '…' : ''}</div>
    </div>
    <div style="text-align:center;padding:20px">
      <div class="spinner" style="display:inline-block;width:24px;height:24px;border:3px solid var(--border);border-top-color:var(--primary);border-radius:50%;animation:spin 1s linear infinite"></div>
      <p style="color:var(--text-muted);margin-top:12px">در حال اندازه‌گیری خودکار ${State.kpis.filter(k=>k.active).length} KPI...</p>
    </div>
  `, '');
  document.head.insertAdjacentHTML('beforeend', '<style>@keyframes spin{to{transform:rotate(360deg)}}</style>');

  try {
    const res = await api('/kpis/measure/' + id);
    const measurements = res.measurements || [];
    const overall = res.overall_score;
    const level = res.level;
    const cf = res.critical_fail;
    const scoreColor = cf ? 'var(--danger)' : overall >= 85 ? 'var(--success)' : overall >= 60 ? 'var(--warning)' : 'var(--danger)';

    openModal('ارزیابی خودکار', `
      <div style="background:var(--surface-2);padding:12px;border-radius:8px;margin-bottom:14px">
        <b>${esc(interaction.subject)}</b>
        <div style="color:var(--text-muted);font-size:12px;margin-top:4px">
          کارشناس: ${esc(agent?.name || '-')} | مشتری: ${esc(customer?.name || '-')}
        </div>
      </div>

      <div style="text-align:center;padding:20px;background:var(--surface-2);border-radius:10px;margin-bottom:16px">
        <div style="font-size:48px;font-weight:800;color:${scoreColor}">${overall.toFixed(1)}</div>
        <div style="color:var(--text-muted);font-size:14px;margin-top:4px">${esc(level)}</div>
        ${cf ? '<div class="pill pill-bad" style="margin-top:8px">شکست بحرانی</div>' : ''}
      </div>

      <h3 style="margin:0 0 10px;font-size:14px">جزئیات اندازه‌گیری KPI</h3>
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
        <label>یادداشت ارزیاب (اختیاری)</label>
        <textarea id="autoScoreNotes" placeholder="نکات تکمیلی شما"></textarea>
      </div>
    `, `<button class="btn btn-primary" id="saveAutoScore">ذخیره در داشبورد</button>
        <button class="btn" data-action="close-modal">انصراف</button>`);

    $('#saveAutoScore').addEventListener('click', async () => {
      const btn = $('#saveAutoScore');
      btn.disabled = true; btn.textContent = 'در حال ذخیره...';
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
        toast(`امتیاز ${saved.overall_score.toFixed(1)} (${saved.level}) ذخیره شد`);
      } catch (e) {
        toast(e.message, 'error');
        btn.disabled = false; btn.textContent = 'ذخیره در داشبورد';
      }
    });
  } catch (e) {
    closeModal();
    if (e.message.includes('هیچ KPI فعالی')) {
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
  openModal('جزئیات تعامل', `
    <div class="field"><b>کارشناس:</b> ${esc(agent?.name || '-')} <span class="pill pill-info">${esc(agent?.department || '')}</span></div>
    <div class="field"><b>مشتری:</b> ${esc(customer?.name || '-')} <span class="pill pill-muted">${esc(customer?.segment || '')}</span></div>
    <div class="field"><b>کانال:</b> <span class="pill pill-muted">${esc(i.channel)}</span> &nbsp; <b>تاریخ:</b> ${fmtDate(i.created_at)}</div>
    <div class="field"><b>موضوع:</b> ${esc(i.subject)}</div>
    <div class="field" style="background:var(--surface-2);padding:12px;border-radius:8px">
      <b style="display:block;margin-bottom:6px">متن مکالمه:</b>
      <div style="white-space:pre-wrap">${esc(i.transcript)}</div>
    </div>
    ${s ? `
      <div style="margin-top:14px;padding:16px;background:var(--surface-2);border-radius:8px;text-align:center">
        <div style="font-size:36px;font-weight:800;color:${s.critical_fail ? 'var(--danger)' : 'var(--success)'}">${Number(s.overall_score).toFixed(1)}</div>
        <div style="color:var(--text-muted)">${esc(s.level)}</div>
        ${s.critical_fail ? '<div class="pill pill-bad" style="margin-top:8px">شکست بحرانی</div>' : ''}
        ${s.notes ? `<div style="margin-top:10px;text-align:right;color:var(--text-muted);font-size:12px">${esc(s.notes)}</div>` : ''}
        ${s.evaluator ? `<div style="margin-top:6px;color:var(--text-muted);font-size:11px">ارزیاب: ${esc(s.evaluator)}</div>` : ''}
      </div>
    ` : ''}
  `, `<button class="btn btn-primary" id="fromViewScore">${s ? 'بازبینی' : 'ارزیابی خودکار'}</button>
      <button class="btn" data-action="close-modal">بستن</button>`);
  $('#fromViewScore')?.addEventListener('click', () => { closeModal(); openAutoScore(id); });
}

// ============ Recommendations (Risk Queue) ============
function renderRecommendations() {
  const list = $('#recommendationList');
  if (!list) return;
  if (!State.recommendations.length) {
    list.innerHTML = '<div class="list-item" style="text-align:center;color:var(--text-muted);padding:40px">همه تعاملات ارزیابی شده‌اند. عالی!</div>';
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
            کارشناس: ${esc(r.agent_name || '-')} | مشتری: ${esc(r.customer_name || '-')} | کانال: ${esc(r.channel)}
          </div>
        </div>
        <div style="text-align:center">
          <div class="risk-score" style="color:${color}">${r.risk_score.toFixed(0)}</div>
          <div style="font-size:10px;color:var(--text-muted)">ریسک</div>
          ${priorityPill(r.priority)}
        </div>
      </div>
      <div class="rec-body">
        <div style="margin-bottom:8px"><b>دلایل ریسک:</b></div>
        <ul class="factors">
          ${reasons.map(f => {
            if (typeof f === 'string') {
              return `<li><span class="factor-pill">${esc(f.split(':')[0] || 'عامل')}</span> <span style="color:var(--text-muted);font-size:12px">${esc(f.includes(':') ? f.split(':').slice(1).join(':') : f)}</span></li>`;
            }
            return `<li><span class="factor-pill">${esc(f.label || f.code || '')}</span> <span style="color:var(--text-muted);font-size:12px">${esc(f.reason || '')}</span></li>`;
          }).join('')}
        </ul>
        <div style="margin-top:8px;padding:8px;background:var(--surface-2);border-radius:6px">
          <b>اقدام پیشنهادی:</b> ${esc(r.suggested_action)}
        </div>
        <div style="margin-top:10px">
          <button class="btn btn-sm btn-primary" data-rec-auto="${r.interaction_id}">ارزیابی خودکار</button>
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
      <td><b>${esc(a.name)}</b></td>
      <td><span class="pill pill-info">${esc(a.department)}</span></td>
      <td>${esc(a.position)}</td>
      <td>${a.active ? '<span class="pill pill-good">فعال</span>' : '<span class="pill pill-muted">غیرفعال</span>'}</td>
      <td class="row-actions">
        <button class="btn btn-sm" data-toggle-agent="${a.id}" data-active="${!a.active}">${a.active ? 'غیرفعال' : 'فعال'}</button>
        <button class="btn btn-sm btn-primary" data-agent-report="${a.id}">گزارش</button>
      </td>
    </tr>
  `).join('') || `<tr><td colspan="5" style="text-align:center;padding:40px;color:var(--text-muted)">کارشناسی یافت نشد</td></tr>`;
  tbody.querySelectorAll('[data-toggle-agent]').forEach(b => b.addEventListener('click', () => toggleAgent(b.dataset.toggleAgent, b.dataset.active === 'true')));
  tbody.querySelectorAll('[data-agent-report]').forEach(b => b.addEventListener('click', () => { switchTab('report'); setTimeout(() => { $('#reportAgent').value = b.dataset.agentReport; renderReport(); }, 50); }));
}

async function toggleAgent(id, active) {
  try {
    await api('/agents/' + id, { method: 'PATCH', body: JSON.stringify({ active }) });
    State.agents = await api('/agents'); cacheSet('agents', State.agents);
    renderAgents();
    toast(active ? 'فعال شد' : 'غیرفعال شد');
  } catch (e) { toast(e.message, 'error'); }
}

function openNewAgent() {
  openModal('ثبت کارشناس', `
    <div class="field"><label>نام و نام خانوادگی</label><input id="aName" placeholder="مثال: علی رضایی"></div>
    <div class="field"><label>واحد</label><select id="aDept"><option>بانک</option><option>بیمه</option><option>عمومی</option></select></div>
    <div class="field"><label>سمت</label><input id="aPos" placeholder="مثال: کارشناس ارشد"></div>
  `, `<button class="btn btn-primary" id="saveAgent">ذخیره</button>
      <button class="btn" data-action="close-modal">انصراف</button>`);
  $('#saveAgent').addEventListener('click', async () => {
    try {
      await api('/agents', { method: 'POST', body: JSON.stringify({
        name: $('#aName').value.trim(), department: $('#aDept').value, position: $('#aPos').value.trim()
      })});
      State.loaded.agents = false;
      closeModal(); await loadAgents();
      toast('کارشناس ثبت شد');
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
      <td class="row-actions">
        <button class="btn btn-sm" data-del-customer="${c.id}">حذف</button>
      </td>
    </tr>
  `).join('') || `<tr><td colspan="5" style="text-align:center;padding:40px;color:var(--text-muted)">مشتری یافت نشد</td></tr>`;
  tbody.querySelectorAll('[data-del-customer]').forEach(b => b.addEventListener('click', async () => {
    if (!confirm('حذف شود؟')) return;
    try { await api('/customers/' + b.dataset.delCustomer, { method: 'DELETE' }); State.loaded.customers = false; await loadCustomers(); toast('حذف شد'); }
    catch (e) { toast(e.message, 'error'); }
  }));
}

function openNewCustomer() {
  openModal('ثبت مشتری', `
    <div class="field"><label>نام</label><input id="cName"></div>
    <div class="field"><label>تلفن</label><input id="cPhone"></div>
    <div class="field"><label>محصول</label><input id="cProduct" placeholder="مثال: تسهیلات"></div>
    <div class="field"><label>بخش</label><select id="cSeg"><option>عادی</option><option>VIP</option><option>شرکتی</option></select></div>
    <div class="field"><label>یادداشت</label><textarea id="cNotes"></textarea></div>
  `, `<button class="btn btn-primary" id="saveCustomer">ذخیره</button>
      <button class="btn" data-action="close-modal">انصراف</button>`);
  $('#saveCustomer').addEventListener('click', async () => {
    try {
      await api('/customers', { method: 'POST', body: JSON.stringify({
        name: $('#cName').value.trim(), phone: $('#cPhone').value.trim(),
        product_type: $('#cProduct').value.trim(), segment: $('#cSeg').value,
        notes: $('#cNotes').value.trim()
      })});
      State.loaded.customers = false;
      closeModal(); await loadCustomers(); toast('مشتری ثبت شد');
    } catch (e) { toast(e.message, 'error'); }
  });
}

// ============ New Interaction ============
function openNewInteraction() {
  if (!State.agents.length || !State.customers.length) {
    Promise.all([loadAgents(), loadCustomers()]).then(openNewInteraction);
    return;
  }
  openModal('ثبت تعامل جدید', `
    <div class="field"><label>کارشناس</label><select id="iAgent">${State.agents.filter(a => a.active).map(a => `<option value="${a.id}">${esc(a.name)} — ${esc(a.department)}</option>`).join('')}</select></div>
    <div class="field"><label>مشتری</label><select id="iCust">${State.customers.map(c => `<option value="${c.id}">${esc(c.name)} — ${esc(c.product_type)}</option>`).join('')}</select></div>
    <div class="field"><label>کانال</label><select id="iCh"><option>تلفن</option><option>حضوری</option><option>ایمیل</option><option>چت</option><option>پیامک</option></select></div>
    <div class="field"><label>موضوع</label><input id="iSub"></div>
    <div class="field"><label>متن مکالمه</label><textarea id="iTr" style="min-height:120px" placeholder="مثال: سلام. مشتری با عصبانیت شکایت کرد که ..."></textarea></div>
  `, `<button class="btn btn-primary" id="saveInteraction">ثبت</button>
      <button class="btn" data-action="close-modal">انصراف</button>`);
  $('#saveInteraction').addEventListener('click', async () => {
    try {
      const sub = $('#iSub').value.trim();
      const tr = $('#iTr').value.trim();
      if (!sub || !tr) throw new Error('موضوع و متن الزامی است');
      await api('/interactions', { method: 'POST', body: JSON.stringify({
        agent_id: $('#iAgent').value, customer_id: $('#iCust').value,
        channel: $('#iCh').value, subject: sub, transcript: tr, tags: []
      })});
      State.loaded.interactions = false;
      State.loaded.dashboard = false;
      State.loaded.rec = false;
      closeModal(); await loadInteractions(); await loadDashboard();
      toast('تعامل ثبت شد');
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
      <div style="margin-bottom:16px">هیچ KPI تعریف نشده است</div>
      <button class="btn btn-primary" id="emptySeedKpis">بارگذاری ۷ KPI پیشفرض فارسی</button>
    </div>`;
    $('#emptySeedKpis')?.addEventListener('click', seedKpis);
    return;
  }
  const kindLabel = {
    keyword_count: 'تعداد کلمه کلیدی',
    keyword_presence: 'وجود کلمه',
    text_length: 'طول متن',
    keyword_ratio: 'نسبت کلمه',
    response_time: 'زمان پاسخ',
    manual_range: 'دستی',
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
      ${k.pattern ? `<div style="font-size:11px;color:var(--text-muted);margin:4px 0"><b>الگو:</b> <code>${esc(k.pattern)}</code></div>` : ''}
      ${k.threshold != null ? `<div style="font-size:11px;color:var(--text-muted);margin:4px 0"><b>آستانه:</b> ${esc(k.threshold)}</div>` : ''}
      <div class="kpi-card-actions">
        <button class="btn btn-sm" data-toggle-kpi="${k.id}" data-active="${!k.active}">${k.active ? 'غیرفعال' : 'فعال'}</button>
        <button class="btn btn-sm" data-del-kpi="${k.id}">حذف</button>
      </div>
    </div>
  `).join('');
  list.querySelectorAll('[data-toggle-kpi]').forEach(b => b.addEventListener('click', async () => {
    try {
      const active = b.dataset.active === 'true';
      await api('/kpis/' + b.dataset.toggleKpi, { method: 'PATCH', body: JSON.stringify({ active }) });
      State.kpis = await api('/kpis'); renderKpis();
      toast(active ? 'فعال شد' : 'غیرفعال شد');
    } catch (e) { toast(e.message, 'error'); }
  }));
  list.querySelectorAll('[data-del-kpi]').forEach(b => b.addEventListener('click', async () => {
    if (!confirm('این KPI حذف شود؟')) return;
    try {
      await api('/kpis/' + b.dataset.delKpi, { method: 'DELETE' });
      State.kpis = await api('/kpis'); renderKpis();
      toast('حذف شد');
    } catch (e) { toast(e.message, 'error'); }
  }));
}

async function seedKpis() {
  try {
    const result = await api('/kpis/seed', { method: 'POST' });
    State.kpis = await api('/kpis');
    renderKpis();
    toast(`${result.length} KPI بارگذاری شد`);
  } catch (e) { toast(e.message, 'error'); }
}

$('#seedKpisBtn')?.addEventListener('click', seedKpis);
$('#newKpiBtn')?.addEventListener('click', openNewKpi);

function openNewKpi() {
  openModal('تعریف KPI جدید', `
    <div class="field"><label>کد (انگلیسی، یکتا)</label><input id="kCode" placeholder="مثل: empathy_keywords"></div>
    <div class="field"><label>نام نمایشی (فارسی)</label><input id="kName" placeholder="مثل: همدلی"></div>
    <div class="field"><label>نوع سنجش</label>
      <select id="kKind">
        <option value="keyword_count">تعداد کلمه کلیدی (هر چند کلمه با کاما)</option>
        <option value="keyword_presence">وجود یا عدم وجود کلمه</option>
        <option value="text_length">طول متن (تعداد کلمه)</option>
        <option value="keyword_ratio">نسبت کلمه به کل (برای کلمات منفی/مثبت)</option>
        <option value="response_time">زمان پاسخ (میلی‌ثانیه)</option>
        <option value="manual_range">دستی (امتیاز توسط ارزیاب)</option>
      </select>
    </div>
    <div class="field"><label>توضیح</label><textarea id="kDesc" placeholder="چه چیزی اندازه‌گیری می‌شود؟"></textarea></div>
    <div class="field"><label>الگو (برای keyword_*, در غیر این صورت خالی)</label>
      <input id="kPattern" placeholder="مثل: سلام,درود,صبح بخیر">
    </div>
    <div class="field"><label>آستانه (برای keyword_count, text_length)</label>
      <input id="kThreshold" type="number" step="0.1" placeholder="مثل: 2.0 برای 'حداقل ۲ بار'">
    </div>
    <div class="field"><label>وزن (۰-۱۰۰)</label><input id="kWeight" type="number" min="0" max="100" step="1" value="10"></div>
    <div class="field"><label><input type="checkbox" id="kCritical"> شکست بحرانی (اگر نمره کمتر از ۶۰ باشد، کل interaction شکست می‌خورد)</label></div>
  `, `<button class="btn btn-primary" id="saveKpi">ذخیره KPI</button>
      <button class="btn" data-action="close-modal">انصراف</button>`);
  $('#saveKpi').addEventListener('click', async () => {
    const code = $('#kCode').value.trim();
    const name = $('#kName').value.trim();
    const weight = parseFloat($('#kWeight').value);
    if (!code || !name) { toast('کد و نام الزامی است', 'error'); return; }
    if (isNaN(weight) || weight < 0 || weight > 100) { toast('وزن باید ۰-۱۰۰ باشد', 'error'); return; }
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
      toast('KPI اضافه شد');
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
      <td class="row-actions">
        ${x.status === 'باز' ? `<button class="btn btn-sm btn-success" data-resolve-issue="${x.id}">CAPA</button>` : '<span style="color:var(--text-muted);font-size:12px">بسته شد</span>'}
      </td>
    </tr>`;
  }).join('') || `<tr><td colspan="6" style="text-align:center;padding:40px;color:var(--text-muted)">ایرادی یافت نشد</td></tr>`;
  tbody.querySelectorAll('[data-resolve-issue]').forEach(b => b.addEventListener('click', () => openResolve(b.dataset.resolveIssue)));
}

$('#iStatus')?.addEventListener('change', renderIssues);
$('#iSeverity')?.addEventListener('change', renderIssues);

function openResolve(id) {
  openModal('اقدام اصلاحی (CAPA)', `
    <div class="field"><label>علت ریشه‌ای</label><textarea id="capRoot" placeholder="چرا این اتفاق افتاد؟"></textarea></div>
    <div class="field"><label>اقدام اصلاحی</label><textarea id="capAct" placeholder="چه اقدامی انجام می‌شود؟"></textarea></div>
  `, `<button class="btn btn-success" id="saveCapa">بستن و ثبت</button>
      <button class="btn" data-action="close-modal">انصراف</button>`);
  $('#saveCapa').addEventListener('click', async () => {
    try {
      const root = $('#capRoot').value.trim();
      const act = $('#capAct').value.trim();
      if (!root) throw new Error('علت ریشه‌ای الزامی است');
      await api('/issues/' + id + '/resolve', { method: 'PATCH', body: JSON.stringify({ root_cause: root, corrective_action: act })});
      State.loaded.issues = false;
      closeModal(); await loadIssues(); await loadDashboard();
      toast('ایراد بسته شد');
    } catch (e) { toast(e.message, 'error'); }
  });
}

// ============ Report ============
function populateReportAgents() {
  const sel = $('#reportAgent');
  if (!sel) return;
  sel.innerHTML = '<option value="">انتخاب کارشناس...</option>' + State.agents.map(a => `<option value="${a.id}">${esc(a.name)} — ${esc(a.department)}</option>`).join('');
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
      { label: 'میانگین امتیاز', value: Number(r.average_score || 0).toFixed(1), cls: 'success' },
      { label: 'تعداد ارزیابی', value: r.scored_interactions, cls: 'primary' },
      { label: 'شکست بحرانی', value: r.critical_failures, cls: r.critical_failures > 0 ? 'danger' : 'success' },
    ];
    $('#reportBody').innerHTML = `
      <div class="kpi-grid">
        ${kpis.map(k => `<div class="kpi"><div class="kpi-label">${k.label}</div><div class="kpi-value ${k.cls}">${k.value}</div></div>`).join('')}
      </div>
      <div class="card">
        <table class="data-table">
          <thead><tr><th>تاریخ</th><th>امتیاز</th><th>سطح</th><th>وضعیت</th></tr></thead>
          <tbody>${(r.scores || []).map(s => `
            <tr>
              <td>${fmtDate(s.created_at)}</td>
              <td><b>${Number(s.overall_score).toFixed(1)}</b></td>
              <td>${scorePill(s.overall_score, s.critical_fail)}</td>
              <td>${s.critical_fail ? '<span class="pill pill-bad">بحرانی</span>' : '<span class="pill pill-good">عادی</span>'}</td>
            </tr>
          `).join('') || `<tr><td colspan="4" style="text-align:center;padding:40px;color:var(--text-muted)">هنوز ارزیابی نشده</td></tr>`}</tbody>
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

// ============ Boot ============
if (State.token) {
  enterApp();
}
