// CRM Quality Inspector - frontend v0.3
// Features: lazy load, localStorage cache, CSV export, Chart.js trend, full-text search

const $ = (s) => document.querySelector(s);
const $$ = (s) => Array.from(document.querySelectorAll(s));

const TOKEN_KEY = 'crm_qi_token';
const USER_KEY = 'crm_qi_user';
const CACHE_KEY = 'crm_qi_cache_v1';
const CACHE_TTL = 60_000; // 60s

const State = {
  token: localStorage.getItem(TOKEN_KEY) || null,
  user: JSON.parse(localStorage.getItem(USER_KEY) || 'null'),
  agents: [],
  customers: [],
  interactions: [],
  rubrics: [],
  scores: {},
  issues: [],
  recommendations: [],
  dashboard: null,
  loaded: { agents: false, customers: false, interactions: false, issues: false, rubrics: false, dashboard: false, rec: false },
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
    const entry = obj[key];
    if (!entry) return null;
    if (Date.now() - entry.t > CACHE_TTL) return null;
    return entry.v;
  } catch { return null; }
}

function cacheSet(key, value) {
  try {
    const raw = localStorage.getItem(CACHE_KEY) || '{}';
    const obj = JSON.parse(raw);
    obj[key] = { t: Date.now(), v: value };
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
function toast(msg, type = 'success') {
  const t = $('#toast');
  t.textContent = msg;
  t.className = 'toast show ' + type;
  setTimeout(() => t.classList.remove('show'), 3000);
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
  State.dashboard = null; State.recommendations = [];
  Object.keys(State.loaded).forEach(k => State.loaded[k] = false);
  $('#loginScreen').classList.remove('hidden');
  $('#appShell').classList.add('hidden');
}

// ============ App ============
async function enterApp() {
  $('#loginScreen').classList.add('hidden');
  $('#appShell').classList.remove('hidden');
  $('#userBadge').textContent = State.user?.username || '';
  // Load only dashboard at boot
  await loadDashboard();
  switchTab('dashboard');
}

async function loadDashboard() {
  if (State.loaded.dashboard) {
    renderDashboard();
    return;
  }
  const cached = cacheGet('dashboard');
  if (cached) { State.dashboard = cached; State.loaded.dashboard = true; renderDashboard(); return; }
  try {
    const d = await api('/reports/dashboard');
    State.dashboard = d;
    State.loaded.dashboard = true;
    cacheSet('dashboard', d);
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
  // lazy-load scores in background
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

async function loadRubrics() {
  if (State.loaded.rubrics) { renderRubrics(); return; }
  State.rubrics = await api('/rubrics'); State.loaded.rubrics = true;
  renderRubrics();
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
  $(`.nav-item[data-tab="${tab}"]`).classList.add('active');
  $('#topbarTitle').textContent = {
    dashboard: 'داشبورد', interactions: 'تعاملات', agents: 'کارشناسان',
    customers: 'مشتریان', recommendations: 'پیشنهادهای QA', issues: 'ایرادات',
    rubrics: 'استانداردها', report: 'گزارش کارشناس',
  }[tab] || tab;
  // Lazy load
  if (tab === 'dashboard') loadDashboard();
  else if (tab === 'interactions') loadInteractions();
  else if (tab === 'agents') loadAgents();
  else if (tab === 'customers') loadCustomers();
  else if (tab === 'issues') loadIssues();
  else if (tab === 'rubrics') loadRubrics();
  else if (tab === 'recommendations') loadRecommendations();
  else if (tab === 'report') { loadAgents().then(populateReportAgents); }
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
  // Trend chart (uses local scores if loaded; otherwise placeholder)
  renderTrendChart();
}

function renderTrendChart() {
  const canvas = $('#trendChart');
  if (!canvas) return;
  if (State.trendChart) { State.trendChart.destroy(); State.trendChart = null; }
  const scores = Object.values(State.scores);
  if (scores.length < 2) {
    canvas.getContext('2d').fillText('برای نمایش نمودار، حداقل ۲ ارزیابی لازم است', 10, 30);
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
      fill: true,
      tension: 0.3,
      pointRadius: 4,
      pointBackgroundColor: '#8b5cf6',
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

  // populate agent filter
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
        <button class="btn btn-sm btn-primary" data-score="${i.id}">${s ? 'بازبینی' : 'ارزیابی'}</button>
        <button class="btn btn-sm" data-view="${i.id}">مشاهده</button>
      </td>
    </tr>`;
  }).join('') || `<tr><td colspan="7" style="text-align:center;padding:40px;color:var(--text-muted)">تعاملی یافت نشد</td></tr>`;

  tbody.querySelectorAll('[data-score]').forEach(b => b.addEventListener('click', () => openScore(b.dataset.score)));
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
    rows.push([
      i.id, fmtDate(i.created_at), agent?.name || '', customer?.name || '',
      i.channel, i.subject,
      s ? s.overall_score : '',
      s ? s.level : '',
      s ? (s.critical_fail ? 'بله' : 'خیر') : '',
      s?.notes || '',
    ]);
  }
  const csv = '\uFEFF' + rows.map(r => r.map(c => `"${String(c).replace(/"/g, '""')}"`).join(',')).join('\n');
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url; a.download = `interactions-${new Date().toISOString().slice(0,10)}.csv`;
  a.click(); URL.revokeObjectURL(url);
  toast('فایل CSV دانلود شد');
}

async function openScore(id) {
  const interaction = State.interactions.find(i => i.id === id);
  if (!interaction) return;
  await loadRubrics();
  const customer = State.customers.find(c => c.id === interaction.customer_id);
  const r = State.rubrics.find(x => x.active && (!x.product_type || x.product_type === customer?.product_type))
    || State.rubrics.find(x => x.active) || State.rubrics[0];
  if (!r) { toast('استانداردی یافت نشد', 'error'); return; }
  // Make sure score is loaded
  if (!State.scores[id]) {
    try { State.scores[id] = await api('/scoring/' + id); } catch {}
  }
  const old = State.scores[id];
  const criteria = r.criteria || [];
  openModal('ارزیابی کیفیت تعامل', `
    <div style="background:var(--surface-2);padding:12px;border-radius:8px;margin-bottom:14px">
      <b>${esc(interaction.subject)}</b>
      <div style="color:var(--text-muted);font-size:12px;margin-top:4px">${esc(interaction.transcript)}</div>
    </div>
    <div id="criteriaList">
      ${criteria.map((c, n) => `
        <div class="score-item">
          <div class="score-row">
            <span class="score-title">${esc(c.title)} ${c.critical ? '<span class="pill pill-bad">بحرانی</span>' : ''}</span>
            <span class="score-weight">وزن ${c.weight}%</span>
          </div>
          <div class="score-desc">${esc(c.description)}</div>
          <input type="range" class="score-range" data-score-range="${n}" min="0" max="100" value="${old?.dimension_scores?.[n] ?? 80}">
          <div class="score-val" id="sv${n}">${old?.dimension_scores?.[n] ?? 80}</div>
        </div>
      `).join('')}
    </div>
    <div class="field"><label>یادداشت ارزیاب (اختیاری)</label><textarea id="scoreNotes" placeholder="نکات یا توضیحات تکمیلی">${esc(old?.notes || '')}</textarea></div>
  `, `<button class="btn btn-primary" id="submitScoreBtn">ثبت ارزیابی</button>
      <button class="btn" data-action="close-modal">انصراف</button>`);

  document.querySelectorAll('[data-score-range]').forEach(r => {
    r.addEventListener('input', e => {
      document.getElementById('sv' + e.target.dataset.scoreRange).textContent = e.target.value;
    });
  });
  $('#submitScoreBtn').addEventListener('click', async () => {
    const scores = Array.from(document.querySelectorAll('[data-score-range]')).map(r => Number(r.value));
    const submitBtn = $('#submitScoreBtn');
    submitBtn.disabled = true;
    submitBtn.textContent = 'در حال ثبت...';
    try {
      const res = await api('/scoring/score', {
        method: 'POST',
        body: JSON.stringify({
          interaction_id: id, rubric_id: r.id, scores,
          evaluator: State.user?.username,
          notes: $('#scoreNotes').value.trim(),
        })
      });
      State.scores[id] = res;
      invalidateCache();
      closeModal();
      renderInteractions();
      renderTrendChart();
      toast(`امتیاز ثبت شد: ${res.overall_score} (${res.level})`);
    } catch (e) {
      toast(e.message, 'error');
      submitBtn.disabled = false;
      submitBtn.textContent = 'ثبت ارزیابی';
    }
  });
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
      </div>
    ` : ''}
  `, `<button class="btn btn-primary" id="fromViewScore">${s ? 'بازبینی ارزیابی' : 'ارزیابی'}</button>
      <button class="btn" data-action="close-modal">بستن</button>`);
  $('#fromViewScore')?.addEventListener('click', () => { closeModal(); openScore(id); });
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
    <div class="field"><label>متن مکالمه</label><textarea id="iTr" style="min-height:120px"></textarea></div>
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
      closeModal(); await loadInteractions(); await loadDashboard();
      toast('تعامل ثبت شد');
    } catch (e) { toast(e.message, 'error'); }
  });
}

// ============ Recommendations ============
function renderRecommendations() {
  const list = $('#recommendationList');
  if (!list) return;
  if (!State.recommendations.length) {
    list.innerHTML = '<div class="list-item" style="text-align:center;color:var(--text-muted)">پیشنهادی وجود ندارد</div>';
    return;
  }
  list.innerHTML = State.recommendations.map(r => {
    const agent = State.agents.find(a => a.id === r.agent_id);
    return `<div class="list-item">
      <div class="list-item-header">
        <span class="list-item-title">${esc(r.customer_name || 'تعامل')}</span>
        <span class="pill ${r.priority === 'بالا' ? 'pill-warn' : 'pill-info'}">اولویت: ${esc(r.priority)}</span>
      </div>
      <div class="list-item-body">
        <div><b>کارشناس:</b> ${esc(agent?.name || '-')} <span class="pill pill-muted">${esc(agent?.department || '')}</span></div>
        <div><b>دلیل:</b> ${esc(r.reason)}</div>
        <div><b>اقدام پیشنهادی:</b> ${esc(r.suggested_action)}</div>
        <div style="margin-top:8px">
          <button class="btn btn-sm btn-primary" data-rec-score="${r.interaction_id}">ارزیابی تعامل</button>
        </div>
      </div>
    </div>`;
  }).join('');
  list.querySelectorAll('[data-rec-score]').forEach(b => b.addEventListener('click', () => { switchTab('interactions'); setTimeout(() => openScore(b.dataset.recScore), 100); }));
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
    const agent = State.agents.find(a => a.id === x.agent_id);
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

// ============ Rubrics ============
function renderRubrics() {
  const list = $('#rubricList');
  if (!list) return;
  if (!State.rubrics.length) {
    list.innerHTML = '<div class="list-item" style="text-align:center;color:var(--text-muted)">استانداردی یافت نشد</div>';
    return;
  }
  list.innerHTML = State.rubrics.map(r => `
    <div class="list-item">
      <div class="list-item-header">
        <span class="list-item-title">${esc(r.name)} <span class="pill ${r.active ? 'pill-good' : 'pill-muted'}" style="margin-right:6px">${r.active ? 'فعال' : 'غیرفعال'}</span> <span class="pill pill-info">نسخه ${r.version}</span></span>
        <span class="list-item-meta">${esc(r.department)} — ${esc(r.product_type || 'همه محصولات')}</span>
      </div>
      ${r.criteria.map(c => `
        <div style="padding:10px 0;border-bottom:1px solid var(--border)">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:4px">
            <span><b>${esc(c.title)}</b> ${c.critical ? '<span class="pill pill-bad">بحرانی</span>' : ''}</span>
            <span class="score-weight">${c.weight}%</span>
          </div>
          <div style="color:var(--text-muted);font-size:12px;margin-bottom:6px">${esc(c.description)}</div>
          <div class="bar-track"><div class="bar-fill" style="width:${c.weight}%;background:${c.critical ? 'var(--danger)' : 'var(--primary)'}"></div></div>
        </div>
      `).join('')}
    </div>
  `).join('');
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
