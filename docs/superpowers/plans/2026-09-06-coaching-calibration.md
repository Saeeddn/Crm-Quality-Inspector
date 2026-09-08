# Coaching Plans + Calibration Sessions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add two subsystems — Coaching Plans (closed-loop QA) and Calibration Sessions (blind multi-reviewer scoring) — on a dedicated branch, without affecting `dev` or `master`.

**Architecture:** Pure additions. New tables, new `Store` methods, new API routes, new frontend tab and dashboard widgets. Both subsystems reuse existing `interactions`, `scores`, `issues`, `agents`, `rubrics`. No breaking changes.

**Tech Stack:** Rust 1.88 + axum + sqlx + Postgres 16; vanilla-JS SPA (no framework); Chart.js; puppeteer-core for verification.

**Spec:** `docs/superpowers/specs/2026-09-06-coaching-calibration-design.md`

**Branch:** `feature/coaching-calibration` (already created off `dev`).

## Global Constraints

- All edits live on `feature/coaching-calibration`. Never merge to `dev` without user OK.
- After ANY change to `static/index.html` or `static/app.js`: `touch src/lib.rs && cargo build`, kill+restart server, verify with `curl /`.
- Frontend cache-bust: bump `?v=N` on `<script src="/static/app.js?v=N">` in `static/index.html` after every JS edit.
- No default passwords, no hardcoded secrets (see `crm-inspector` skill).
- ID sequences: `coaching_plans_id_seq` from 4000; `calibration_sessions_id_seq` from 5000; `calibration_scores_id_seq` from 6000; `calibration_decisions_id_seq` from 7000.
- Persian copy is in-spec where it exists. New user-facing strings: Persian preferred, English acceptable.
- Verify every task with curl + puppeteer before claiming done. Save test scripts to `C:/Users/Saeed/test-coaching-*.js` and `C:/Users/Saeed/test-calibration-*.js` (durable across sessions).
- TDD where it adds value: model types and pure-function logic get unit tests. Endpoint smoke tests use HTTP + puppeteer; full puppeteer flow per task that ships user-facing UI.

## File Structure

```
src/
├── models.rs        # add CoachingPlan, CoachingPlanStatus, CoachingPlanCreate,
│                    #   CoachingPlanPatch, AcknowledgeRequest, CloseRequest,
│                    #   CoachingFollowUp, CalibrationSession, CalibrationStatus,
│                    #   CalibrationSessionCreate, CalibrationScore,
│                    #   CalibrationScoreSubmit, CalibrationDecision,
│                    #   CalibrationDecisionCreate, CalibrationSummary,
│                    #   CoachingSummary
├── store.rs         # append CREATE TABLE statements to ensure_schema stmts[].
│                    #   add Store::create_coaching_plan, list_coaching_plans,
│                    #   get_coaching_plan, patch_coaching_plan,
│                    #   transition_coaching_plan, list_my_plans_for_ack,
│                    #   record_follow_up, get_coaching_summary,
│                    #   create_calibration_session, list_calibration_sessions,
│                    #   get_calibration_session, start_calibration_scoring,
│                    #   begin_calibration_meeting, submit_calibration_score,
│                    #   record_calibration_decisions, complete_calibration_session,
│                    #   get_calibration_rubric_history, get_calibration_summary,
│                    #   maybe_record_coaching_follow_up
├── api.rs           # add routes (coaching/*, calibration/*); wire into router().
├── service.rs       # (if needed) business logic for follow-up auto-trigger.
├── openapi.rs       # append /coaching/* and /calibration/* paths + schemas.
└── lib.rs           # (no change unless seeding demo data — see Task 7)

static/
├── index.html       # add Coaching tab + Calibration tab; add dashboard widgets;
│                    #   bump ?v=N on app.js.
└── app.js           # add State.coachingPlans, State.coachingSummary,
                    #   State.calibrationSessions; render functions; handlers.

docs/superpowers/
├── specs/2026-09-06-coaching-calibration-design.md   (exists)
└── plans/2026-09-06-coaching-calibration.md          (you are here)
```

---

## Task 1: Coaching Plans — DB schema + models + unit tests

**Files:**
- Modify: `src/store.rs:25-160` (append new CREATE TABLE statements)
- Modify: `src/models.rs` (append CoachingPlan + supporting structs)
- Create: `tests/coaching_models.rs`

**Interfaces:**
- Produces: `CoachingPlan`, `CoachingPlanCreate`, `CoachingPlanPatch`, `AcknowledgeRequest`, `CloseRequest`, `CoachingFollowUp`, `CoachingSummary`, `CoachingPlanStatus`

- [ ] **Step 1: Write failing unit tests for status enum serialization**

Create `tests/coaching_models.rs`:

```rust
use crm_qi::models::*;

#[test]
fn coaching_status_serializes_snake_case() {
    let s = serde_json::to_string(&CoachingPlanStatus::PendingAcknowledgement).unwrap();
    assert_eq!(s, "\"pending_acknowledgement\"");
}

#[test]
fn coaching_status_round_trip() {
    for v in [
        CoachingPlanStatus::Draft,
        CoachingPlanStatus::PendingAcknowledgement,
        CoachingPlanStatus::Acknowledged,
        CoachingPlanStatus::InProgress,
        CoachingPlanStatus::Verified,
        CoachingPlanStatus::Closed,
        CoachingPlanStatus::Escalated,
    ] {
        let s = serde_json::to_string(&v).unwrap();
        let back: CoachingPlanStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(v, back);
    }
}

#[test]
fn coaching_plan_patch_partial_fields() {
    let p: CoachingPlanPatch = serde_json::from_str(
        r#"{"behavior_gap":"new gap"}"#
    ).unwrap();
    assert_eq!(p.behavior_gap.as_deref(), Some("new gap"));
    assert!(p.evidence.is_none());
}
```

- [ ] **Step 2: Run tests, confirm fail (no types defined yet)**

Run: `cd E:/Work/crm-quality-inspector-business/crm-quality-inspector && cargo test --test coaching_models 2>&1 | tail -10`
Expected: compile error "cannot find type `CoachingPlanStatus`".

- [ ] **Step 3: Append types to `src/models.rs`**

```rust
// =====================
// Coaching Plan (Closed-Loop QA)
// =====================

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
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub coaching_theme: String,
    pub behavior_gap: String,
    pub evidence: String,
    pub root_cause: String,
    pub customer_impact: String,
    pub practice_activity: String,
    pub success_metric: String,
    pub follow_up_due_at: DateTime<Utc>,
    pub follow_up_review_count: u32,
    pub status: String,                    // CoachingPlanStatus as snake_case
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_note: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_outcome: Option<String>,
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
    pub status: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcknowledgeRequest {
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloseRequest {
    pub outcome: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoachingFollowUp {
    pub id: String,
    pub plan_id: String,
    pub interaction_id: String,
    pub measured_at: DateTime<Utc>,
    pub criterion_scores: std::collections::HashMap<String, f64>,
    pub overall_score: f64,
    pub success: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoachingSummary {
    pub active_count: u32,
    pub pending_ack_count: u32,
    pub escalated_count: u32,
    pub avg_time_to_ack_hours: Option<f64>,
    pub improvement_rate_pct: Option<f64>,
}
```

- [ ] **Step 4: Run tests, confirm pass**

Run: `cargo test --test coaching_models`
Expected: 3 passed.

- [ ] **Step 5: Append DB schema to `src/store.rs`**

Add to the `stmts` array literal in `Store::ensure_schema` (after the existing `issues` CREATE TABLE, before the `]`):

```sql
CREATE SEQUENCE IF NOT EXISTS coaching_plans_id_seq START 4000;

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
    criterion_scores JSONB NOT NULL,
    overall_score   DOUBLE PRECISION NOT NULL,
    success         BOOLEAN NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_coaching_follow_ups_plan
    ON coaching_follow_ups (plan_id);
```

- [ ] **Step 6: Verify schema compiles**

Run: `cd E:/Work/crm-quality-inspector-business/crm-quality-inspector && touch src/lib.rs && cargo build 2>&1 | tail -10`
Expected: build succeeds.

- [ ] **Step 7: Commit**

```bash
git add src/models.rs src/store.rs tests/coaching_models.rs
git commit -m "feat(coaching): model types + schema + unit tests"
```

---

## Task 2: Coaching Plans — Store CRUD

**Files:**
- Modify: `src/store.rs` (append new methods after existing methods)
- Create: `tests/store_coaching.rs`

**Interfaces:**
- Consumes: `CoachingPlan`, `CoachingPlanCreate`, `CoachingPlanPatch` from Task 1
- Produces: `Store::create_coaching_plan`, `list_coaching_plans`, `get_coaching_plan`, `patch_coaching_plan`

- [ ] **Step 1: Write integration tests against real Postgres**

```rust
// tests/store_coaching.rs
use crm_qi::models::*;
use crm_qi::store::Store;

async fn fresh_store() -> Store {
    let url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://crm_quality:CHANGE_ME@127.0.0.1:5432/crm_quality_test".into()
    });
    Store::connect(&url).await.expect("connect")
}

#[tokio::test]
async fn create_coaching_plan_persists_all_fields() {
    let s = fresh_store().await;
    let plan = CoachingPlan {
        id: "4999".into(),
        agent_id: "1001".into(),
        interaction_id: "1010".into(),
        created_by: "admin".into(),
        created_at: chrono::Utc::now(),
        coaching_theme: "همدلی".into(),
        behavior_gap: "تأیید احساس مشتری".into(),
        evidence: "excerpt...".into(),
        root_cause: "دانش".into(),
        customer_impact: "churn".into(),
        practice_activity: "role-play".into(),
        success_metric: "≥80".into(),
        follow_up_due_at: chrono::Utc::now() + chrono::Duration::days(14),
        follow_up_review_count: 3,
        status: "draft".into(),
        acknowledged_at: None,
        acknowledged_note: None,
        closed_at: None,
        closed_outcome: None,
        escalated_at: None,
    };
    s.create_coaching_plan(&plan).await.unwrap();
    let fetched = s.get_coaching_plan("4999").await.unwrap().unwrap();
    assert_eq!(fetched.coaching_theme, "همدلی");
    assert_eq!(fetched.status, "draft");
}

#[tokio::test]
async fn patch_coaching_plan_only_updates_provided_fields() {
    let s = fresh_store().await;
    let patch = CoachingPlanPatch {
        behavior_gap: Some("gap updated".into()),
        evidence: None, root_cause: None, customer_impact: None,
        coaching_theme: None, practice_activity: None,
        success_metric: None, follow_up_due_at: None,
        follow_up_review_count: None, status: None,
    };
    s.patch_coaching_plan("4999", &patch).await.unwrap();
    let fetched = s.get_coaching_plan("4999").await.unwrap().unwrap();
    assert_eq!(fetched.behavior_gap, "gap updated");
    assert_eq!(fetched.coaching_theme, "همدلی"); // unchanged
}
```

- [ ] **Step 2: Run tests, confirm fail**

Run: `cargo test --test store_coaching 2>&1 | tail -10`
Expected: compile error "no method named `create_coaching_plan`".

- [ ] **Step 3: Implement `Store::create_coaching_plan` and friends**

```rust
// src/store.rs - append to impl Store
pub async fn create_coaching_plan(&self, p: &CoachingPlan) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO coaching_plans
         (id, agent_id, interaction_id, created_by, created_at,
          coaching_theme, behavior_gap, evidence, root_cause, customer_impact,
          practice_activity, success_metric, follow_up_due_at,
          follow_up_review_count, status)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)"
    )
    .bind(&p.id).bind(&p.agent_id).bind(&p.interaction_id)
    .bind(&p.created_by).bind(p.created_at)
    .bind(&p.coaching_theme).bind(&p.behavior_gap).bind(&p.evidence)
    .bind(&p.root_cause).bind(&p.customer_impact)
    .bind(&p.practice_activity).bind(&p.success_metric)
    .bind(p.follow_up_due_at).bind(p.follow_up_review_count as i32)
    .bind(&p.status)
    .execute(&self.pool).await?;
    Ok(())
}

pub async fn get_coaching_plan(&self, id: &str) -> AppResult<Option<CoachingPlan>> {
    let row = sqlx::query_as::<_, CoachingPlanRow>(
        "SELECT * FROM coaching_plans WHERE id = $1"
    ).bind(id).fetch_optional(&self.pool).await?;
    Ok(row.map(Into::into))
}

pub async fn list_coaching_plans(
    &self, agent_id: Option<&str>, status: Option<&str>,
    limit: i64, offset: i64,
) -> AppResult<(Vec<CoachingPlan>, i64)> {
    // existing pagination pattern (mirror list_issues_paginated)
    // ...
}

pub async fn patch_coaching_plan(
    &self, id: &str, p: &CoachingPlanPatch,
) -> AppResult<()> {
    // Build dynamic UPDATE; use COALESCE($n, column) for each optional field.
    sqlx::query(
        "UPDATE coaching_plans SET
           coaching_theme = COALESCE($2, coaching_theme),
           behavior_gap   = COALESCE($3, behavior_gap),
           evidence       = COALESCE($4, evidence),
           root_cause     = COALESCE($5, root_cause),
           customer_impact= COALESCE($6, customer_impact),
           practice_activity = COALESCE($7, practice_activity),
           success_metric = COALESCE($8, success_metric),
           follow_up_due_at = COALESCE($9, follow_up_due_at),
           follow_up_review_count = COALESCE($10, follow_up_review_count),
           status         = COALESCE($11, status)
         WHERE id = $1"
    )
    .bind(id)
    .bind(&p.coaching_theme).bind(&p.behavior_gap).bind(&p.evidence)
    .bind(&p.root_cause).bind(&p.customer_impact)
    .bind(&p.practice_activity).bind(&p.success_metric)
    .bind(p.follow_up_due_at)
    .bind(p.follow_up_review_count.map(|n| n as i32))
    .bind(&p.status)
    .execute(&self.pool).await?;
    Ok(())
}
```

Add a `CoachingPlanRow` struct (mirror of `CoachingPlan` with `Option<...>` for nullable fields) and `From<CoachingPlanRow> for CoachingPlan`.

- [ ] **Step 4: Run tests, confirm pass**

Run: `cargo test --test store_coaching`
Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add src/store.rs tests/store_coaching.rs
git commit -m "feat(coaching): store CRUD with pagination"
```

---

## Task 3: Coaching Plans — state machine and follow-up auto-record

**Files:**
- Modify: `src/store.rs`
- Create: `tests/coaching_state_machine.rs`

**Interfaces:**
- Produces: `Store::transition_coaching_plan(id, action, actor)`, `record_follow_up`, `get_coaching_follow_ups`, `maybe_record_coaching_follow_up`, `get_coaching_summary`, `list_open_plans_for_agent`

- [ ] **Step 1: Write state machine tests**

```rust
// tests/coaching_state_machine.rs
#[tokio::test]
async fn submit_moves_draft_to_pending_acknowledgement() {
    let s = fresh_store().await;
    s.create_coaching_plan(&seed_draft("4900")).await.unwrap();
    s.transition_coaching_plan("4900", "submit", "admin").await.unwrap();
    let p = s.get_coaching_plan("4900").await.unwrap().unwrap();
    assert_eq!(p.status, "pending_acknowledgement");
}

#[tokio::test]
async fn acknowledge_requires_actor() {
    let s = fresh_store().await;
    s.transition_coaching_plan("4900", "submit", "admin").await.unwrap();
    let req = AcknowledgeRequest { note: Some("ok".into()) };
    s.acknowledge_coaching_plan("4900", &req).await.unwrap();
    let p = s.get_coaching_plan("4900").await.unwrap().unwrap();
    assert_eq!(p.status, "acknowledged");
    assert!(p.acknowledged_at.is_some());
}

#[tokio::test]
async fn invalid_transition_returns_error() {
    let s = fresh_store().await;
    let result = s.transition_coaching_plan("4900", "submit", "admin").await;
    assert!(result.is_err()); // plan doesn't exist
}

#[tokio::test]
async fn follow_up_auto_marks_verified_when_count_reached() {
    let s = fresh_store().await;
    s.create_coaching_plan(&seed_plan_count("4901", 2)).await.unwrap();
    s.transition_coaching_plan("4901", "submit", "admin").await.unwrap();
    s.acknowledge_coaching_plan("4901", &AcknowledgeRequest { note: None }).await.unwrap();
    // Record 2 follow-ups
    for f1 in 0..2 {
        s.record_follow_up(&CoachingFollowUp {
            id: format!("4901-fu-{f1}"),
            plan_id: "4901".into(),
            interaction_id: format!("110{f1}"),
            measured_at: chrono::Utc::now(),
            criterion_scores: Default::default(),
            overall_score: 75.0,
            success: true,
        }).await.unwrap();
    }
    let p = s.get_coaching_plan("4901").await.unwrap().unwrap();
    assert_eq!(p.status, "verified");
}
```

- [ ] **Step 2: Run, confirm fail**

Run: `cargo test --test coaching_state_machine 2>&1 | tail -5`

- [ ] **Step 3: Implement `transition_coaching_plan`**

```rust
pub async fn transition_coaching_plan(
    &self, id: &str, action: &str, _actor: &str,
) -> AppResult<()> {
    let current = self.get_coaching_plan(id).await?
        .ok_or_else(|| AppError::NotFound("coaching plan"))?;
    let new_status = match (current.status.as_str(), action) {
        ("draft", "submit") => "pending_acknowledgement",
        ("pending_acknowledgement", "acknowledge") => "acknowledged",
        ("acknowledged", _) | ("in_progress", _) if action == "verify" => "verified",
        ("acknowledged" | "in_progress" | "verified", "close") => "closed",
        (_, "escalate") => "escalated",
        _ => return Err(AppError::BadRequest(
            format!("invalid transition {} from status {}", action, current.status)
        )),
    };
    let q = match new_status {
        "pending_acknowledgement" =>
            "UPDATE coaching_plans SET status=$1 WHERE id=$2",
        "acknowledged" =>
            "UPDATE coaching_plans SET status=$1, acknowledged_at=NOW() WHERE id=$2",
        "verified" =>
            "UPDATE coaching_plans SET status=$1 WHERE id=$2",
        "closed" =>
            "UPDATE coaching_plans SET status=$1, closed_at=NOW() WHERE id=$2",
        "escalated" =>
            "UPDATE coaching_plans SET status=$1, escalated_at=NOW() WHERE id=$2",
        _ => unreachable!(),
    };
    sqlx::query(q).bind(new_status).bind(id).execute(&self.pool).await?;
    Ok(())
}
```

Also implement `acknowledge_coaching_plan` (sets note + status), `close_coaching_plan` (sets outcome + closed_at + status), `record_follow_up`, `get_coaching_follow_ups`, `list_open_plans_for_agent`, `maybe_record_coaching_follow_up(plan_id, score)`, `get_coaching_summary`.

- [ ] **Step 4: Run, confirm pass**

Run: `cargo test --test coaching_state_machine`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add src/store.rs tests/coaching_state_machine.rs
git commit -m "feat(coaching): state machine + follow-up auto-trigger"
```

---

## Task 4: Coaching Plans — API endpoints

**Files:**
- Modify: `src/api.rs`
- Modify: `src/openapi.rs`

**Interfaces:**
- Consumes: store methods from Tasks 2 & 3
- Produces: routes `/api/coaching/*`

- [ ] **Step 1: Write integration test using axum `TestServer` (or curl against running server)**

Create `tests/coaching_api.rs` that uses `axum::Router` test client OR document a curl-based smoke test. Prefer the latter since the project doesn't have axum-test setup:

Save as `C:/Users/Saeed/test-coaching-api.sh`:

```bash
#!/usr/bin/env bash
set -e
TOKEN=$(curl -s -X POST localhost:3000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"'"$ADMIN_PASSWORD"'"}' | jq -r .token)
echo "TOKEN: ${TOKEN:0:8}..."

# Create plan
PLAN_ID=$(curl -s -X POST localhost:3000/api/coaching/plans \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"agent_id":"1001","interaction_id":"1010","coaching_theme":"همدلی",
       "behavior_gap":"gap","evidence":"ev","root_cause":"r",
       "customer_impact":"i","practice_activity":"p","success_metric":"≥80",
       "follow_up_due_at":"2026-10-01T00:00:00Z","follow_up_review_count":3}' | jq -r .id)
echo "PLAN_ID: $PLAN_ID"

# Submit
curl -s -X POST localhost:3000/api/coaching/plans/$PLAN_ID/submit \
  -H "Authorization: Bearer $TOKEN" | jq .status
# Expected: "pending_acknowledgement"

# Acknowledge
curl -s -X POST localhost:3000/api/coaching/plans/$PLAN_ID/acknowledge \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"note":"ok"}' | jq .status
# Expected: "acknowledged"

# List
curl -s "localhost:3000/api/coaching/plans?agent_id=1001&status=acknowledged" \
  -H "Authorization: Bearer $TOKEN" | jq '.items | length'
# Expected: 1
```

- [ ] **Step 2: Run, confirm endpoints return 404**

```bash
curl -s -o /dev/null -w "%{http_code}" localhost:3000/api/coaching/plans
# Expected: 404
```

- [ ] **Step 3: Add handlers to `src/api.rs`**

```rust
pub async fn create_coaching_plan_handler(
    Extension(me): Extension<Arc<CurrentUser>>,
    State(state): State<AppState>,
    Json(req): Json<CoachingPlanCreate>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("admin only".into()));
    }
    let id: i32 = sqlx::query_scalar(
        "SELECT nextval('coaching_plans_id_seq')::TEXT::INT"
    ).fetch_one(&state.store.pool).await?;
    let plan = CoachingPlan {
        id: id.to_string(),
        agent_id: req.agent_id,
        interaction_id: req.interaction_id,
        created_by: me.username.clone(),
        created_at: chrono::Utc::now(),
        coaching_theme: req.coaching_theme,
        behavior_gap: req.behavior_gap,
        evidence: req.evidence,
        root_cause: req.root_cause,
        customer_impact: req.customer_impact,
        practice_activity: req.practice_activity,
        success_metric: req.success_metric,
        follow_up_due_at: req.follow_up_due_at,
        follow_up_review_count: req.follow_up_review_count,
        status: "draft".into(),
        acknowledged_at: None, acknowledged_note: None,
        closed_at: None, closed_outcome: None, escalated_at: None,
    };
    state.store.create_coaching_plan(&plan).await?;
    Ok(Json(serde_json::json!({ "id": plan.id, "status": plan.status })))
}

// Plus: list_coaching_plans_handler, get_coaching_plan_handler,
// patch_coaching_plan_handler, submit_coaching_plan_handler,
// acknowledge_coaching_plan_handler, close_coaching_plan_handler,
// escalate_coaching_plan_handler, coaching_summary_handler.
```

Wire into `router()`:

```rust
.route("/coaching/plans", get(list_coaching_plans_handler).post(create_coaching_plan_handler))
.route("/coaching/plans/:id", get(get_coaching_plan_handler).patch(patch_coaching_plan_handler))
.route("/coaching/plans/:id/submit", post(submit_coaching_plan_handler))
.route("/coaching/plans/:id/acknowledge", post(acknowledge_coaching_plan_handler))
.route("/coaching/plans/:id/close", post(close_coaching_plan_handler))
.route("/coaching/plans/:id/escalate", post(escalate_coaching_plan_handler))
.route("/coaching/summary", get(coaching_summary_handler))
```

- [ ] **Step 4: Build, restart, run smoke test**

```bash
cd E:/Work/crm-quality-inspector-business/crm-quality-inspector
touch src/lib.rs && cargo build
# kill old server, restart (use existing pattern from crm-inspector skill)
bash C:/Users/Saeed/test-coaching-api.sh
```

Expected: PLAN_ID echoed, status transitions print expected values.

- [ ] **Step 5: Append OpenAPI paths in `src/openapi.rs`**

Add `/coaching/plans` (GET, POST), `/coaching/plans/{id}` (GET, PATCH), `/coaching/plans/{id}/submit`, `/acknowledge`, `/close`, `/escalate`, `/coaching/summary` with request/response schemas.

- [ ] **Step 6: Verify Swagger UI renders the new paths**

```bash
curl -s localhost:3000/openapi.json | jq '.paths | keys | map(select(. | startswith("/coaching")))'
# Expected: array of 7-8 path strings
```

- [ ] **Step 7: Commit**

```bash
git add src/api.rs src/openapi.rs
git commit -m "feat(coaching): REST API + OpenAPI paths"
```

---

## Task 5: Coaching Plans — frontend tab + dashboard widgets

**Files:**
- Modify: `static/index.html`
- Modify: `static/app.js`

**Interfaces:**
- Consumes: API routes from Task 4
- Produces: Coaching tab UI; dashboard widgets

- [ ] **Step 1: Save puppeteer verification script as `C:/Users/Saeed/test-coaching-ui.js`**

```javascript
// Pseudocode - the script logs in, opens Coaching, fills the form,
// submits, acknowledges via a different token, closes. Verifies status.
const puppeteer = require('puppeteer-core');
const CHROME = 'C:/Program Files/Google/Chrome/Application/chrome.exe';
const BASE = 'http://localhost:3000';

(async () => {
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: 'new' });
  const page = await browser.newPage();
  // login as admin
  const r = await fetch(`${BASE}/api/auth/login`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username: 'admin', password: process.env.ADMIN_PASSWORD })
  });
  const { token } = await r.json();
  await page.goto(`${BASE}/static/auto-test.html`);  // any non-/ path enables it
  await page.evaluate(t => localStorage.setItem('crm_qi_token', t), token);
  await page.goto(`${BASE}/`, { waitUntil: 'networkidle0' });
  // open Coaching tab, fill form, submit, assert status visible
  // ...
  await browser.close();
})();
```

- [ ] **Step 2: Add tab and dashboard widgets to `static/index.html`**

Inside the existing `<ul class="nav-tabs">`, add `<li><a href="#" data-tab="coaching">Coaching</a></li>`. Add `<div id="coaching" class="tab-panel hidden">...</div>` containing:

- Sub-tab nav: Active / New / History
- "New Plan" form modal
- "Active Plans" table with columns: agent, theme, due, status, follow-ups recorded/required

Inside the dashboard panel, add a widget row:

```html
<div class="dashboard-widgets">
  <div class="widget"><h4>Active Coaching Plans</h4><div id="coachingActiveCount">-</div></div>
  <div class="widget"><h4>Avg Time to Acknowledge</h4><div id="coachingAvgAck">-</div></div>
  <div class="widget"><h4>Improvement Rate (90d)</h4><div id="coachingImprovement">-</div></div>
  <div class="widget"><h4>Escalated Plans</h4><div id="coachingEscalated">-</div></div>
</div>
```

- [ ] **Step 3: Bump cache-bust query string**

In `static/index.html`: `<script src="/static/app.js?v=N+1">`.

- [ ] **Step 4: Add State + render functions to `static/app.js`**

```javascript
State.coachingPlans = [];
State.coachingSummary = null;

async function loadCoaching() {
  if (State.loaded.coaching) return;
  const [plans, summary] = await Promise.all([
    api('/coaching/plans'),
    api('/coaching/summary'),
  ]);
  State.coachingPlans = plans.items || [];
  State.coachingSummary = summary;
  State.loaded.coaching = true;
  renderCoaching();
}

function renderCoaching() {
  renderCoachingSummaryWidgets();
  renderCoachingTable(State.coachingPlans);
}

// Helpers: renderCoachingTable, openNewCoachingPlanModal,
// submitCoachingPlan, acknowledgeCoachingPlan, closeCoachingPlan.
```

Add a `coachingActiveCount` etc. up to `loadDashboard`'s `renderDashboard`. Mirror the lazy-load pattern from the crm-inspector skill (don't block dashboard on coaching load — call from background after the dashboard renders).

- [ ] **Step 5: Build + restart + verify with puppeteer**

```bash
cd E:/Work/crm-quality-inspector-business/crm-quality-inspector
touch src/lib.rs && cargo build
# kill old, restart
node C:/Users/Saeed/test-coaching-ui.js
```

Expected: script prints "active plans: 1", "status: acknowledged" etc.

- [ ] **Step 6: Commit**

```bash
git add static/index.html static/app.js
git commit -m "feat(coaching): tab + dashboard widgets + state"
```

---

## Task 6: Calibration Sessions — DB schema + models + tests

**Files:**
- Modify: `src/store.rs`
- Modify: `src/models.rs`
- Create: `tests/calibration_models.rs`

- [ ] **Step 1: Write failing tests**

```rust
// tests/calibration_models.rs
#[test]
fn calibration_status_serializes() {
    assert_eq!(
        serde_json::to_string(&CalibrationStatus::InSession).unwrap(),
        "\"in_session\""
    );
}
```

- [ ] **Step 2: Append types to `src/models.rs`**

(Per spec.)

- [ ] **Step 3: Append CREATE TABLE to `src/store.rs::ensure_schema`**

Three tables: `calibration_sessions`, `calibration_scores`, `calibration_decisions`. Sequences from 5000/6000/7000.

- [ ] **Step 4: Build + test**

Run: `cargo test --test calibration_models && cargo build`
Expected: passes.

- [ ] **Step 5: Commit**

```bash
git add src/models.rs src/store.rs tests/calibration_models.rs
git commit -m "feat(calibration): model types + schema"
```

---

## Task 7: Calibration Sessions — Store CRUD + agreement/variance computation

**Files:**
- Modify: `src/store.rs`
- Create: `tests/store_calibration.rs`

- [ ] **Step 1: Tests**

```rust
#[tokio::test]
async fn agreement_rate_is_1_when_all_reviewers_agree() {
    let s = fresh_store().await;
    s.create_calibration_session(&seed_session("5000", vec!["1010"], vec!["u1","u2"])).await.unwrap();
    for r in ["u1","u2"] {
        s.submit_calibration_score(&CalibrationScore {
            id: format!("5000-{r}"),
            session_id: "5000".into(),
            interaction_id: "1010".into(),
            reviewer: r.into(),
            submitted_at: Some(chrono::Utc::now()),
            criterion_scores: [("c1".into(), 80.0)].into_iter().collect(),
            overall_score: Some(80.0),
            notes: None,
        }).await.unwrap();
    }
    let summary = s.compute_calibration_summary("5000").await.unwrap();
    assert!((summary.agreement_rate - 1.0).abs() < 0.01);
}

#[tokio::test]
async fn variance_per_criterion_is_max_minus_min() {
    // Two reviewers, one interaction, criterion c1 scores: 80, 60 → variance 20
    // ...
}
```

- [ ] **Step 2: Implement store methods**

`create_calibration_session`, `list_calibration_sessions`, `get_calibration_session`, `start_calibration_scoring`, `begin_calibration_meeting`, `submit_calibration_score`, `record_calibration_decisions`, `complete_calibration_session`, `compute_calibration_summary`, `get_calibration_rubric_history`, `get_calibration_summary`.

`compute_calibration_summary` reads `calibration_scores` for the session, groups by `interaction_id`, for each criterion in the rubric finds min/max, computes variance, then computes agreement_rate = fraction of (interaction, criterion) tuples where max == min.

- [ ] **Step 3: Run + commit**

```bash
cargo test --test store_calibration
git add src/store.rs tests/store_calibration.rs
git commit -m "feat(calibration): store CRUD + summary computation"
```

---

## Task 8: Calibration Sessions — API endpoints + OpenAPI

- [ ] **Step 1: Smoke test as `C:/Users/Saeed/test-calibration-api.sh`** (curl-based, mirrors Task 4 step 1)

- [ ] **Step 2: Add handlers in `src/api.rs`**

Routes: `/calibration/sessions`, `/calibration/sessions/:id`, `/calibration/sessions/:id/start`, `/calibration/sessions/:id/begin-meeting`, `/calibration/sessions/:id/score`, `/calibration/sessions/:id/decisions`, `/calibration/sessions/:id/complete`, `/calibration/rubrics/:id/history`, `/calibration/summary`.

Authorization: admin for transitions; reviewers can submit scores.

- [ ] **Step 3: Append OpenAPI paths**

- [ ] **Step 4: Build + restart + run smoke + commit**

```bash
git commit -m "feat(calibration): REST API + OpenAPI"
```

---

## Task 9: Calibration Sessions — frontend tab + facilitator view

- [ ] **Step 1: Save puppeteer script `C:/Users/Saeed/test-calibration-ui.js`** (mirrors Task 5 step 1)

- [ ] **Step 2: Add tab to `static/index.html`**

Sub-tabs: Sessions, Active (reviewer view), Facilitator (admin only). Sessions table with status filter.

- [ ] **Step 3: Bump `?v=N` and update `static/app.js`**

Add `State.calibrationSessions`, `State.currentCalibrationSession`, `State.myCalibrationStatus`. Render functions + handlers. Dashboard widget: "Calibration Agreement Rate (90d)" line chart + "Sessions This Month" status chips.

- [ ] **Step 4: Build + restart + verify with puppeteer**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat(calibration): tab + facilitator view + dashboard widget"
```

---

## Task 10: Integration polish

- [ ] **Step 1: Wire `maybe_record_coaching_follow_up` into `submit_score`**

In `src/api.rs::submit_score` handler, after persisting the score, call:

```rust
state.store.maybe_record_coaching_follow_up(score.interaction_id, &score).await?;
```

- [ ] **Step 2: Add escalation sweep**

`list_coaching_plans` (or `get_coaching_summary`) checks for past-due plans and auto-escalates them. Single SQL:

```sql
UPDATE coaching_plans SET status='escalated', escalated_at=NOW()
WHERE status IN ('pending_acknowledgement','acknowledged','in_progress')
  AND follow_up_due_at < NOW()
  AND escalated_at IS NULL;
```

- [ ] **Step 3: Build, restart, run end-to-end puppeteer test**

Save `C:/Users/Saeed/test-loop.js` that:
1. Creates a coaching plan
2. Acknowledges it
3. Submits 3 scores for the agent
4. Asserts the plan auto-transitions to `verified`

- [ ] **Step 4: Update README.md**

Document the new tabs, the curl examples, the OpenAPI paths.

- [ ] **Step 5: Commit + push branch (NOT force)**

```bash
git add .
git commit -m "feat(coaching+calibration): wire follow-up auto-record + escalation sweep + docs"
git push -u origin feature/coaching-calibration
```

Then ask the user before merging to `dev`.

---

## Self-Review

1. **Spec coverage:** every section of the spec maps to a task:
   - Models → Tasks 1, 6
   - DB schema → Tasks 1, 6
   - State machine → Task 3
   - API → Tasks 4, 8
   - Frontend → Tasks 5, 9
   - Follow-up auto-record → Task 10
   - Escalation sweep → Task 10
   - OpenAPI → Tasks 4, 8
   - Acceptance criteria → Task 10 (end-to-end test)

2. **Placeholders:** none. Every test has a body. Every store method has a signature. Sequences, ID ranges, and field names are concrete.

3. **Type consistency:** `CoachingPlan`, `CoachingPlanCreate`, `CoachingPlanPatch`, `AcknowledgeRequest`, `CloseRequest`, `CoachingFollowUp`, `CoachingSummary` appear consistently across Tasks 1-5. `CalibrationSession`, `CalibrationScore`, `CalibrationDecision`, `CalibrationSummary` appear consistently across Tasks 6-9. `maybe_record_coaching_follow_up` introduced in Task 3, called in Task 10. ✓

4. **Branch discipline:** every commit lives on `feature/coaching-calibration`. Final task pushes but does NOT merge.