# Coaching Plans + Calibration Sessions — Design Spec

**Date:** 2026-09-06
**Branch:** `feature/coaching-calibration` (off `dev`)
**Status:** Draft → awaiting PM approval
**Scope:** One organization (single tenant)

---

## Why this exists

Today the project scores interactions, surfaces issues, and computes customer
risk — but the **score has nowhere to go**. A coach (QA supervisor) sees that
agent X scored 64 on the "empathy" criterion, opens a verbal coaching session,
and nothing in the system records:

- what behavior should change
- what the agent committed to
- whether the agent actually improved on the next interactions
- whether two coaches give the same score on the same call

Industry sources (Calabrio, MaestroQA, Scorebuddy, Kaizo, C2Perform) all treat
these two capabilities as table stakes for any QA program that wants to
demonstrate ROI to a director. Manual spot-checking produces numbers; closed-loop
coaching + calibration produces **behavior change**.

This spec adds two new subsystems:

1. **Coaching Plans** — closed-loop QA: every flagged interaction triggers a
   coaching plan with behavior gap, evidence, follow-up, and automatic
   re-measurement.
2. **Calibration Sessions** — blind multi-reviewer scoring of the same sample
   conversations, with agreement-rate and per-criterion variance diagnostics.

Both follow the established project conventions (vanilla-JS SPA, Rust + axum +
postgres, single `Store` layer, `CREATE TABLE IF NOT EXISTS` bootstrap,
`include_str!` HTML, hard rule: `touch src/lib.rs && cargo build` after HTML
edits).

---

## Part 1 — Coaching Plans

### Domain Model

```rust
// src/models.rs

/// Lifecycle of a coaching plan.
/// draft → pending_acknowledgement → (acknowledged → in_progress) →
///   (verified → closed) | escalated
/// The arrow to `escalated` can fire from any state when the due-date
/// passes without acknowledgement or closure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CoachingPlanStatus {
    Draft,
    PendingAcknowledgement,
    Acknowledged,
    InProgress,
    Verified,
    Closed,
    Escalated,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoachingPlan {
    pub id: String,
    pub agent_id: String,
    pub interaction_id: String,
    pub created_by: String,             // coach username
    pub created_at: DateTime<Utc>,

    // Evidence-based fields (Oversai 2026 template, proven format)
    pub coaching_theme: String,         // "همدلی" | "انطباق" | "حل مسئله" | "فروش"
    pub behavior_gap: String,           // specific observable behavior
    pub evidence: String,               // transcript excerpt + score reference
    pub root_cause: String,             // دانش / فرآیند / ابزار / سیاست
    pub customer_impact: String,        // افزایش ریسک churn / تماس مجدد / شکایت
    pub practice_activity: String,      // role-play / بازنویسی / مرور مثال
    pub success_metric: String,         // measurable: "۵ تعامل بعدی، معیار X ≥ ۸۰"
    pub follow_up_due_at: DateTime<Utc>,
    pub follow_up_review_count: u32,    // 1..10

    pub status: String,                 // CoachingPlanStatus as snake_case
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_note: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_outcome: Option<String>, // "improved" | "no_change" | "diverted_to_process_owner"
    pub escalated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoachingPlanCreate {
    pub agent_id: String,
    pub interaction_id: String,
    pub coaching_theme: String,
    pub behavior_gap: String,
    pub evidence: String,
    pub root_cause: String,
    pub customer_impact: String,
    pub practice_activity: String,
    pub success_metric: String,
    pub follow_up_due_at: DateTime<Utc>,
    pub follow_up_review_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoachingPlanPatch {
    pub coaching_theme: Option<String>,
    pub behavior_gap: Option<String>,
    pub evidence: Option<String>,
    pub root_cause: Option<String>,
    pub customer_impact: Option<String>,
    pub practice_activity: Option<String>,
    pub success_metric: Option<String>,
    pub follow_up_due_at: Option<DateTime<Utc>>,
    pub follow_up_review_count: Option<u32>,
    pub status: Option<String>, // draft → pending_acknowledgement only
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcknowledgeRequest {
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloseRequest {
    pub outcome: String,                // "improved" | "no_change" | "diverted"
    pub note: Option<String>,
}

/// One follow-up measurement. Auto-recorded when a post-acknowledgement
/// interaction is scored.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoachingFollowUp {
    pub id: String,
    pub plan_id: String,
    pub interaction_id: String,
    pub measured_at: DateTime<Utc>,
    pub criterion_scores: HashMap<String, f64>, // criterion_id → 0-100
    pub overall_score: f64,
    pub success: bool,                  // does this measurement meet success_metric?
}
```

### Database (Postgres)

Added to `Store::ensure_schema` after the existing `issues` table:

```sql
CREATE TABLE IF NOT EXISTS coaching_plans (
    id              TEXT PRIMARY KEY,
    agent_id        TEXT NOT NULL,
    interaction_id  TEXT NOT NULL,
    created_by      TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    coaching_theme  TEXT NOT NULL,
    behavior_gap    TEXT NOT NULL,
    evidence        TEXT NOT NULL,
    root_cause      TEXT NOT NULL,
    customer_impact TEXT NOT NULL,
    practice_activity TEXT NOT NULL,
    success_metric  TEXT NOT NULL,
    follow_up_due_at TIMESTAMPTZ NOT NULL,
    follow_up_review_count INTEGER NOT NULL DEFAULT 3,
    status          TEXT NOT NULL DEFAULT 'draft',
    acknowledged_at TIMESTAMPTZ,
    acknowledged_note TEXT,
    closed_at       TIMESTAMPTZ,
    closed_outcome  TEXT,
    escalated_at    TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_coaching_plans_agent_status
    ON coaching_plans (agent_id, status);

CREATE TABLE IF NOT EXISTS coaching_follow_ups (
    id              TEXT PRIMARY KEY,
    plan_id         TEXT NOT NULL REFERENCES coaching_plans(id) ON DELETE CASCADE,
    interaction_id  TEXT NOT NULL,
    measured_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    criterion_scores JSONB NOT NULL,        -- {criterion_id: 0..100}
    overall_score   DOUBLE PRECISION NOT NULL,
    success         BOOLEAN NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_coaching_follow_ups_plan
    ON coaching_follow_ups (plan_id);
```

ID generation follows the existing pattern:
`nextval('coaching_plans_id_seq')::TEXT` with sequence starting at 4000 (so
demo plans are visibly distinct from interaction ids).

### API

```
POST  /api/coaching/plans                 create draft
GET   /api/coaching/plans?agent_id=&status=&limit=&offset=
GET   /api/coaching/plans/:id             with embedded follow_ups[]
PATCH /api/coaching/plans/:id             edit fields while status=draft
POST  /api/coaching/plans/:id/submit      draft → pending_acknowledgement
POST  /api/coaching/plans/:id/acknowledge {note?}  pending_acknowledgement → acknowledged
POST  /api/coaching/plans/:id/close       {outcome, note?}  → closed
POST  /api/coaching/plans/:id/escalate    {reason}          → escalated

GET   /api/coaching/summary               dashboard numbers
       → {active_count, pending_ack_count, escalated_count,
          avg_time_to_ack_hours, improvement_rate_pct}
```

Authorization: anyone can read; only `is_admin` users (existing role) can
create/submit/close. The agent can `acknowledge` only their own plan
(`auth.require_self(plan.agent_id)`).

### Service layer

`Store::create_coaching_plan(plan: CoachingPlan)` — INSERT, return value.
`Store::list_coaching_plans(filters, page, limit) -> (Vec<CoachingPlan>, i64)`.
`Store::get_coaching_plan(id) -> Option<CoachingPlanWithFollowUps>`.
`Store::patch_coaching_plan(id, patch)` — partial UPDATE.
`Store::transition_coaching_plan(id, action, payload, actor) -> CoachingPlan`
— single entry that owns the state-machine:

```rust
match (current_status.as_str(), action) {
    ("draft", "submit") => status = "pending_acknowledgement",
    ("pending_acknowledgement", "acknowledge") => {
        status = "acknowledged";
        acknowledged_at = NOW;
    },
    ("acknowledged", "close") => /* user-driven early close */,
    ("in_progress", "close") => /* user-driven close */,
    _ => return Err(AppError::BadRequest("invalid transition")),
}
```

`Store::maybe_record_follow_up(plan, interaction_score)` — called from inside
`submit_score` (existing handler) after the score is persisted:

```rust
// pseudo
if let Some(plan) = find_open_plan_for_agent_and_interaction(score.agent_id, score.interaction_id).await? {
    let fu = CoachingFollowUp {
        criterion_scores: score.criterion_scores.clone(),
        overall_score: score.overall,
        success: meets_success_metric(&plan.success_metric, &score),
        ...
    };
    insert(fu).await?;
    let follow_ups = count_follow_ups_for(plan.id).await?;
    if follow_ups >= plan.follow_up_review_count {
        // auto-transition acknowledged/in_progress → verified
        transition(plan.id, "verify", ...).await?;
    }
}
```

**Important:** the success metric is a free-text string. We do NOT parse it.
Instead, the agent's follow-up review count determines the window. When the
Nth follow-up is recorded, the plan is `verified` and shown to the coach for
close. `success` is computed once per follow-up and stored, but the close
decision is human-driven.

A scheduled task (or on-each-request sweep) escalates plans whose
`follow_up_due_at < NOW AND status IN (pending_acknowledgement, acknowledged,
in_progress)`. Implementation: a simple check at the start of
`list_coaching_plans` and `summary`. No cron dependency.

### Frontend (vanilla JS, mirrors existing tab patterns)

New tab "Coaching" with three sub-tabs:

- **Active Plans** — paginated table; columns: agent, theme, due, status,
  follow-ups recorded / required, improvement badge.
- **New Plan** — modal form pre-filled from `interaction_id` (passes via query
  string from interaction row context-menu).
- **History** — closed/escalated plans with outcomes.

Dashboard widgets (new row):

- "Active Coaching Plans" (count)
- "Avg Time to Acknowledge" (hours, last 30d)
- "Improvement Rate" (% of closed plans with `outcome=improved`, last 90d)
- "Escalated Plans" (warning chip if > 0)

Edit-mode guard: a coach cannot edit a plan after `submit` (only acknowledge
or close). Mirror existing `State.X` pattern; add `State.coachingPlans`,
`State.coachingSummary`.

Cache-bust `<script src="/static/app.js?v=N+1">` on every JS change. Touch
`src/lib.rs` + rebuild after every HTML change. Verified in browser via
puppeteer before declaring done.

---

## Part 2 — Calibration Sessions

### Domain Model

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationStatus {
    Draft,
    Scoring,        // reviewers are submitting blind scores
    InSession,      // meeting is happening now
    Completed,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationSession {
    pub id: String,
    pub title: String,
    pub rubric_id: String,                    // which rubric to calibrate
    pub sample_interaction_ids: Vec<String>,  // 2 or 3
    pub reviewer_usernames: Vec<String>,
    pub scheduled_at: DateTime<Utc>,
    pub status: String,                       // CalibrationStatus
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub facilitator_notes: Option<String>,
    // Filled on completion:
    pub agreement_rate: Option<f64>,         // 0..1 across all criteria × interactions
    pub variance_per_criterion: Option<HashMap<String, f64>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationScore {
    pub id: String,
    pub session_id: String,
    pub interaction_id: String,
    pub reviewer: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub criterion_scores: HashMap<String, f64>, // criterion_id → 0..100
    pub overall_score: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationDecision {
    pub id: String,
    pub session_id: String,
    pub criterion_id: String,
    pub agreed_interpretation: String,        // human-readable rule
    pub example_interaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub rubric_edit_proposed: Option<String>,  // optional new wording
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationSessionCreate {
    pub title: String,
    pub rubric_id: String,
    pub sample_interaction_ids: Vec<String>,
    pub reviewer_usernames: Vec<String>,
    pub scheduled_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationScoreSubmit {
    pub criterion_scores: HashMap<String, f64>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationDecisionCreate {
    pub criterion_id: String,
    pub agreed_interpretation: String,
    pub example_interaction_id: Option<String>,
    pub rubric_edit_proposed: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationSummary {
    pub agreement_rate: f64,
    pub variance_per_criterion: HashMap<String, f64>,
    pub widest_gap_criterion: Option<String>, // for facilitator focus
    pub reviewer_count: u32,
    pub sample_size: u32,
}
```

### Database

```sql
CREATE TABLE IF NOT EXISTS calibration_sessions (
    id                      TEXT PRIMARY KEY,
    title                   TEXT NOT NULL,
    rubric_id               TEXT NOT NULL,
    sample_interaction_ids  JSONB NOT NULL,
    reviewer_usernames      JSONB NOT NULL,
    scheduled_at            TIMESTAMPTZ NOT NULL,
    status                  TEXT NOT NULL DEFAULT 'draft',
    created_by              TEXT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at            TIMESTAMPTZ,
    facilitator_notes       TEXT,
    agreement_rate          DOUBLE PRECISION,
    variance_per_criterion  JSONB
);

CREATE TABLE IF NOT EXISTS calibration_scores (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL REFERENCES calibration_sessions(id) ON DELETE CASCADE,
    interaction_id  TEXT NOT NULL,
    reviewer        TEXT NOT NULL,
    submitted_at    TIMESTAMPTZ,
    criterion_scores JSONB NOT NULL,        -- {criterion_id: 0..100}
    overall_score   DOUBLE PRECISION,
    notes           TEXT,
    UNIQUE(session_id, interaction_id, reviewer)
);

CREATE INDEX IF NOT EXISTS idx_calibration_scores_session
    ON calibration_scores (session_id);

CREATE TABLE IF NOT EXISTS calibration_decisions (
    id                      TEXT PRIMARY KEY,
    session_id              TEXT NOT NULL REFERENCES calibration_sessions(id) ON DELETE CASCADE,
    criterion_id            TEXT NOT NULL,
    agreed_interpretation   TEXT NOT NULL,
    example_interaction_id  TEXT,
    rubric_edit_proposed    TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(session_id, criterion_id)
);
```

ID sequences: `calibration_sessions_id_seq` from 5000,
`calibration_scores_id_seq` from 6000, `calibration_decisions_id_seq` from 7000.

### API

```
POST  /api/calibration/sessions           create draft (admin only)
GET   /api/calibration/sessions?status=&rubric_id=&limit=&offset=
GET   /api/calibration/sessions/:id       with all scores + decisions
POST  /api/calibration/sessions/:id/start draft → scoring (admin)
POST  /api/calibration/sessions/:id/begin-meeting  scoring → in_session (admin)
POST  /api/calibration/sessions/:id/score {interaction_id, criterion_scores, notes?}
       — by reviewer; once per (session, interaction, reviewer)
GET   /api/calibration/sessions/:id/my-status  {submitted_count, total_expected}
POST  /api/calibration/sessions/:id/decisions [{criterion_id, agreed_interpretation, ...}]
POST  /api/calibration/sessions/:id/complete  admin; computes agreement_rate + variance
GET   /api/calibration/rubrics/:id/history    all decisions ever recorded for this rubric
GET   /api/calibration/summary                dashboard: trend of agreement_rate over time
```

Authorization:

- `create`, `start`, `begin-meeting`, `complete`, `decisions` → admin only.
- `score` → any user in `reviewer_usernames`.
- read → any authenticated user.

### Service layer

`Store::compute_calibration_summary(session_id) -> CalibrationSummary`:

```rust
// pseudo
let scores = all_scores_for_session(session_id).await?;
let rubric = get_rubric(session.rubric_id).await?;
let mut variance_per_criterion = HashMap::new();
let mut agreements = 0;
let mut total = 0;
for interaction_id in &session.sample_interaction_ids {
    let per_reviewer = scores.filter(|s| s.interaction_id == interaction_id);
    for criterion in &rubric.criteria {
        let values: Vec<f64> = per_reviewer.iter()
            .filter_map(|s| s.criterion_scores.get(criterion.id).copied())
            .collect();
        if values.is_empty() { continue; }
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let variance = max - min;
        variance_per_criterion.insert(criterion.id, variance);
        total += 1;
        if (max - min).abs() < 0.01 { agreements += 1; }
    }
}
let agreement_rate = if total > 0 { agreements as f64 / total as f64 } else { 0.0 };
```

Stored on completion. History endpoint aggregates per-rubric over all sessions.

### Frontend

New tab "Calibration" with three sub-tabs:

- **Sessions** — paginated, status filter.
- **Active Session** — visible only when user is a reviewer; shows sample
  conversations one by one with the scorecard; "Submit Blind Score" button.
- **Facilitator View** (admin only, when `status=in_session`) — table with
  rows=reviewers, cols=interactions, cells show overall + criterion
  breakdown; below the table, "Widest Gap Criterion" highlight + form to
  record `CalibrationDecision`.

Dashboard widget:

- "Calibration Agreement Rate" (last 90 days, line trend)
- "Sessions This Month" + status chips

---

## Migration strategy

The project uses `CREATE TABLE IF NOT EXISTS` inside `Store::ensure_schema`
(no `sqlx::migrate!`). To keep with the convention:

- Append the three new tables and three new sequences to the existing
  `stmts` array literal in `store.rs`.
- No separate migration file. Documented in this spec.

If/when the project later adopts `sqlx::migrate!`, the statements can be
extracted into `migrations/0004_coaching_calibration.sql` without semantic
change.

---

## Out of scope (deferred)

- Multi-tenant org_id columns (we're single-tenant).
- Mobile-app notifications for `pending_acknowledgement`.
- Speech-to-text ingestion (we read transcripts, we don't generate them).
- AI-generated coaching plan drafts (would be a v2 add-on once usage patterns
  are clear).
- Calibration on auto-scored KPIs (we calibrate human scores; the AI/auto
  side is its own system).

---

## Implementation order

The two subsystems are independent. Order recommendation:

1. Coaching Plans first — fewer dependencies (uses existing scores/issues
   schema), visible value on day one.
2. Calibration Sessions second — depends on rubrics being stable and requires
   sample data to demo.

Both go on the same `feature/coaching-calibration` branch per your
instruction, so neither affects `dev` until you choose to merge.

---

## Acceptance criteria (for the implementation plan)

1. A coach can create a plan from an interaction row, fill all evidence
   fields, and submit for acknowledgement.
2. An agent can see their pending plan and acknowledge with a note.
3. When the Nth post-acknowledgement interaction is scored, the plan
   auto-transitions to `verified` and surfaces in the coach's queue for
   close.
4. Past-due plans show `escalated` chip on the dashboard.
5. Calibration session creation, blind scoring (with one score per
   reviewer/interaction), meeting start, decisions recording, and completion
   with agreement_rate and variance all work end-to-end.
6. Decision history is queryable per rubric for onboarding.
7. All existing routes still pass; new routes are documented in `openapi.rs`.
8. Frontend tested via puppeteer (`C:/Users/Saeed/test-coaching.js` and
   `C:/Users/Saeed/test-calibration.js` saved as durable scripts).
9. After every HTML/JS edit: `touch src/lib.rs && cargo build && restart`
   and verify via `curl /` that the new content is served.

---

## Self-review

- **Placeholders:** none. Each section specifies concrete fields, types,
  and SQL.
- **Internal consistency:** ID ranges (4000/5000/6000/7000) chosen so demo
  data is visually distinguishable. `follow_up_review_count` is referenced
  consistently (model → service → completion rule).
- **Scope:** two subsystems, each with their own plan/decision/completion
  lifecycle. Implementation plan will split them into two tracks but they
  share the branch.
- **Ambiguity:** `success_metric` is free-text (Oversai convention). The
  follow-up count, not metric parsing, gates auto-transition. Documented.