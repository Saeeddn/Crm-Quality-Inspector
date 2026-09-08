use crate::error::{AppError, AppResult};
use crate::models::*;
use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct Store {
    pub pool: PgPool,
}

impl Store {
    pub async fn connect(database_url: &str) -> AppResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|e| AppError::Internal(format!("postgres connect: {e}")))?;
        let s = Self { pool };
        s.ensure_schema().await?;
        Ok(s)
    }

    async fn ensure_schema(&self) -> AppResult<()> {
        let stmts = [
            "CREATE TABLE IF NOT EXISTS users (
                username TEXT PRIMARY KEY,
                password_hash TEXT NOT NULL,
                is_admin BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                department TEXT NOT NULL DEFAULT '',
                position TEXT NOT NULL DEFAULT '',
                active BOOLEAN NOT NULL DEFAULT TRUE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE TABLE IF NOT EXISTS customers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                phone TEXT NOT NULL DEFAULT '',
                product_type TEXT NOT NULL DEFAULT '',
                segment TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE TABLE IF NOT EXISTS interactions (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL DEFAULT '',
                customer_id TEXT NOT NULL DEFAULT '',
                channel TEXT NOT NULL DEFAULT '',
                subject TEXT NOT NULL DEFAULT '',
                transcript TEXT NOT NULL DEFAULT '',
                tags JSONB NOT NULL DEFAULT '[]'::jsonb,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE TABLE IF NOT EXISTS rubrics (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                department TEXT NOT NULL DEFAULT '',
                product_type TEXT,
                channel TEXT,
                version INTEGER NOT NULL DEFAULT 1,
                criteria JSONB NOT NULL DEFAULT '[]'::jsonb,
                active BOOLEAN NOT NULL DEFAULT TRUE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE TABLE IF NOT EXISTS scores (
                id TEXT PRIMARY KEY,
                interaction_id TEXT NOT NULL UNIQUE,
                rubric_id TEXT NOT NULL DEFAULT '',
                overall_score DOUBLE PRECISION NOT NULL DEFAULT 0,
                level TEXT NOT NULL DEFAULT '',
                dimension_scores JSONB NOT NULL DEFAULT '[]'::jsonb,
                critical_fail BOOLEAN NOT NULL DEFAULT FALSE,
                critical_fail_reasons JSONB NOT NULL DEFAULT '[]'::jsonb,
                evaluator TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE TABLE IF NOT EXISTS issues (
                id TEXT PRIMARY KEY,
                interaction_id TEXT NOT NULL DEFAULT '',
                agent_id TEXT NOT NULL DEFAULT '',
                severity TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'باز',
                root_cause TEXT,
                corrective_action TEXT,
                due_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                resolved_at TIMESTAMPTZ
            )",
            "CREATE TABLE IF NOT EXISTS metrics (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT '',
                metric_type TEXT NOT NULL,
                min_val DOUBLE PRECISION,
                max_val DOUBLE PRECISION,
                higher_is_better BOOLEAN NOT NULL DEFAULT TRUE,
                allowed_values JSONB NOT NULL DEFAULT '[]'::jsonb,
                value_scores JSONB NOT NULL DEFAULT '{}'::jsonb,
                required_keywords JSONB NOT NULL DEFAULT '[]'::jsonb,
                scale_min DOUBLE PRECISION,
                scale_max DOUBLE PRECISION,
                critical BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE TABLE IF NOT EXISTS kpis (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                pattern TEXT,
                threshold DOUBLE PRECISION,
                ratio_total_pattern TEXT,
                                weight DOUBLE PRECISION NOT NULL DEFAULT 1.0,
                                critical BOOLEAN NOT NULL DEFAULT FALSE,
                                active BOOLEAN NOT NULL DEFAULT TRUE,
                                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                            )",
                            "CREATE TABLE IF NOT EXISTS coaching_plans (
                                id TEXT PRIMARY KEY,
                                agent_id TEXT NOT NULL,
                                interaction_id TEXT NOT NULL,
                                created_by TEXT NOT NULL,
                                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                                coaching_theme TEXT NOT NULL,
                                behavior_gap TEXT NOT NULL,
                                evidence TEXT NOT NULL,
                                root_cause TEXT NOT NULL,
                                customer_impact TEXT NOT NULL,
                                practice_activity TEXT NOT NULL,
                                success_metric TEXT NOT NULL,
                                follow_up_due_at TIMESTAMPTZ NOT NULL,
                                follow_up_review_count INTEGER NOT NULL DEFAULT 3,
                                status TEXT NOT NULL DEFAULT 'draft',
                                acknowledged_at TIMESTAMPTZ,
                                acknowledged_note TEXT,
                                closed_at TIMESTAMPTZ,
                                closed_outcome TEXT,
                                escalated_at TIMESTAMPTZ
                            )",
                            "CREATE INDEX IF NOT EXISTS idx_coaching_plans_agent_status
                                ON coaching_plans (agent_id, status)",
                            "CREATE TABLE IF NOT EXISTS coaching_follow_ups (
                                id TEXT PRIMARY KEY,
                                plan_id TEXT NOT NULL REFERENCES coaching_plans(id) ON DELETE CASCADE,
                                interaction_id TEXT NOT NULL,
                                measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                                criterion_scores JSONB NOT NULL,
                                overall_score DOUBLE PRECISION NOT NULL,
                                success BOOLEAN NOT NULL
                            )",
                            "CREATE INDEX IF NOT EXISTS idx_coaching_follow_ups_plan
                                ON coaching_follow_ups (plan_id)",
                            // =================== Calibration Sessions ===================
                            "CREATE TABLE IF NOT EXISTS calibration_sessions (
                                id TEXT PRIMARY KEY,
                                name TEXT NOT NULL,
                                status TEXT NOT NULL DEFAULT 'draft',
                                rubric_id TEXT NOT NULL,
                                reviewer_usernames JSONB NOT NULL DEFAULT '[]'::jsonb,
                                sample_interaction_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
                                target_agreement_rate DOUBLE PRECISION,
                                min_reviewers_per_interaction INTEGER NOT NULL DEFAULT 2,
                                deadline_at TIMESTAMPTZ NOT NULL,
                                meeting_started_at TIMESTAMPTZ,
                                facilitator_id TEXT,
                                agreement_rate DOUBLE PRECISION,
                                variance_per_criterion JSONB,
                                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                            )",
                            "CREATE TABLE IF NOT EXISTS calibration_scores (
                                id TEXT PRIMARY KEY,
                                session_id TEXT NOT NULL REFERENCES calibration_sessions(id) ON DELETE CASCADE,
                                interaction_id TEXT NOT NULL,
                                reviewer TEXT NOT NULL,
                                submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                                criterion_scores JSONB NOT NULL,
                                overall_score DOUBLE PRECISION NOT NULL,
                                notes TEXT,
                                UNIQUE(session_id, interaction_id, reviewer)
                            )",
                            "CREATE INDEX IF NOT EXISTS idx_calibration_scores_session
                                ON calibration_scores (session_id)",
                            "CREATE TABLE IF NOT EXISTS calibration_decisions (
                                id TEXT PRIMARY KEY,
                                session_id TEXT NOT NULL REFERENCES calibration_sessions(id) ON DELETE CASCADE,
                                criterion_id TEXT NOT NULL,
                                agreed_interpretation TEXT NOT NULL,
                                example_interaction_id TEXT,
                                rubric_edit_proposed TEXT,
                                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                                UNIQUE(session_id, criterion_id)
                            )",
                        ];
        for sql in stmts {
            sqlx::query(sql).execute(&self.pool).await?;
        }
        // Sequences for clean sequential ids. Each starts at 1000 so the
        // demo data has recognisable ids (1001, 1002, ...). The Store
        // layer reads nextval() and assigns the value as a TEXT id.
        for tbl in ["agents", "customers", "interactions", "rubrics", "scores", "issues", "metrics", "kpis", "coaching_plans", "coaching_follow_ups", "calibration_sessions", "calibration_scores", "calibration_decisions"] {
            sqlx::query(&format!(
                "CREATE SEQUENCE IF NOT EXISTS {tbl}_id_seq START 1000"
            ))
            .execute(&self.pool)
            .await
            .ok();
        }
        Ok(())
    }

    /// Allocate a fresh id from a per-table sequence. Returns a numeric
    /// string ("1001", "1002", ...) used as the entity id.
    pub async fn next_id(&self, seq: &str) -> AppResult<String> {
        let row = sqlx::query(&format!("SELECT nextval('{seq}')::TEXT AS id"))
            .fetch_one(&self.pool)
            .await?;
        let id: String = row.get("id");
        Ok(id)
    }

    // =================== USERS ===================

    pub async fn get_user(&self, username: &str) -> AppResult<Option<User>> {
        let row = sqlx::query("SELECT username, password_hash, is_admin, created_at FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| User {
            username: r.get("username"),
            password_hash: r.get("password_hash"),
            is_admin: r.get("is_admin"),
            created_at: r.get("created_at"),
        }))
    }

    pub async fn put_user(&self, user: &User) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO users (username, password_hash, is_admin, created_at) VALUES ($1, $2, $3, $4)
             ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash, is_admin = EXCLUDED.is_admin"
        )
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(user.is_admin)
        .bind(user.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn user_exists(&self, username: &str) -> AppResult<bool> {
        let row = sqlx::query("SELECT 1 FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.is_some())
    }

    pub async fn list_users(&self) -> AppResult<Vec<String>> {
        let rows = sqlx::query("SELECT username FROM users ORDER BY username")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.get("username")).collect())
    }

    pub async fn list_users_full(&self) -> AppResult<Vec<User>> {
        let rows = sqlx::query("SELECT username, password_hash, is_admin, created_at FROM users ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| User {
            username: r.get("username"),
            password_hash: r.get("password_hash"),
            is_admin: r.get("is_admin"),
            created_at: r.get("created_at"),
        }).collect())
    }

    pub async fn delete_user(&self, username: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn ensure_admin(&self, username: &str, password: &str) -> AppResult<()> {
        let hash = crate::auth::hash_password(password)?;
        if self.user_exists(username).await? {
            // User exists — update password hash to match current ADMIN_PASSWORD env var
            sqlx::query("UPDATE users SET password_hash = $1 WHERE username = $2")
                .bind(&hash)
                .bind(username)
                .execute(&self.pool)
                .await?;
        } else {
            let user = User {
                username: username.into(),
                password_hash: hash,
                is_admin: true,
                created_at: Utc::now(),
            };
            self.put_user(&user).await?;
        }
        Ok(())
    }

    // =================== AGENTS ===================

    pub async fn put_agent(&self, a: &Agent) -> AppResult<Agent> {
        sqlx::query(
            "INSERT INTO agents (id, name, department, position, active, created_at) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, department=EXCLUDED.department, position=EXCLUDED.position, active=EXCLUDED.active"
        )
        .bind(&a.id).bind(&a.name).bind(&a.department).bind(&a.position)
        .bind(a.active).bind(a.created_at)
        .execute(&self.pool).await?;
        Ok(a.clone())
    }

    pub async fn get_agent(&self, id: &str) -> AppResult<Option<Agent>> {
        let row = sqlx::query("SELECT id, name, department, position, active, created_at FROM agents WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| Agent {
            id: r.get("id"),
            name: r.get("name"),
            department: r.get("department"),
            position: r.get("position"),
            active: r.get("active"),
            created_at: r.get("created_at"),
        }))
    }

    pub async fn list_agents(&self) -> AppResult<Vec<Agent>> {
        let rows = sqlx::query("SELECT id, name, department, position, active, created_at FROM agents ORDER BY name")
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| Agent {
            id: r.get("id"),
            name: r.get("name"),
            department: r.get("department"),
            position: r.get("position"),
            active: r.get("active"),
            created_at: r.get("created_at"),
        }).collect())
    }

    pub async fn list_agents_paginated(&self, limit: i64, offset: i64) -> AppResult<(Vec<Agent>, i64)> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents")
            .fetch_one(&self.pool).await?;
        let rows = sqlx::query("SELECT id, name, department, position, active, created_at FROM agents ORDER BY name LIMIT $1 OFFSET $2")
            .bind(limit).bind(offset)
            .fetch_all(&self.pool).await?;
        let out: Vec<Agent> = rows.into_iter().map(|r| Agent {
            id: r.get("id"),
            name: r.get("name"),
            department: r.get("department"),
            position: r.get("position"),
            active: r.get("active"),
            created_at: r.get("created_at"),
        }).collect();
        Ok((out, total))
    }

    pub async fn delete_agent(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM agents WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    // =================== CUSTOMERS ===================

    pub async fn put_customer(&self, x: &Customer) -> AppResult<Customer> {
        sqlx::query(
            "INSERT INTO customers (id, name, phone, product_type, segment, notes, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, phone=EXCLUDED.phone, product_type=EXCLUDED.product_type, segment=EXCLUDED.segment, notes=EXCLUDED.notes"
        )
        .bind(&x.id).bind(&x.name).bind(&x.phone).bind(&x.product_type).bind(&x.segment).bind(&x.notes).bind(x.created_at)
        .execute(&self.pool).await?;
        Ok(x.clone())
    }

    pub async fn get_customer(&self, id: &str) -> AppResult<Option<Customer>> {
        let row = sqlx::query("SELECT id, name, phone, product_type, segment, notes, created_at FROM customers WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| Customer {
            id: r.get("id"),
            name: r.get("name"),
            phone: r.get("phone"),
            product_type: r.get("product_type"),
            segment: r.get("segment"),
            notes: r.get("notes"),
            created_at: r.get("created_at"),
        }))
    }

    pub async fn list_customers(&self) -> AppResult<Vec<Customer>> {
        let rows = sqlx::query("SELECT id, name, phone, product_type, segment, notes, created_at FROM customers ORDER BY name")
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| Customer {
            id: r.get("id"),
            name: r.get("name"),
            phone: r.get("phone"),
            product_type: r.get("product_type"),
            segment: r.get("segment"),
            notes: r.get("notes"),
            created_at: r.get("created_at"),
        }).collect())
    }

    pub async fn list_customers_paginated(&self, limit: i64, offset: i64) -> AppResult<(Vec<Customer>, i64)> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM customers")
            .fetch_one(&self.pool).await?;
        let rows = sqlx::query("SELECT id, name, phone, product_type, segment, notes, created_at FROM customers ORDER BY name LIMIT $1 OFFSET $2")
            .bind(limit).bind(offset)
            .fetch_all(&self.pool).await?;
        let out: Vec<Customer> = rows.into_iter().map(|r| Customer {
            id: r.get("id"),
            name: r.get("name"),
            phone: r.get("phone"),
            product_type: r.get("product_type"),
            segment: r.get("segment"),
            notes: r.get("notes"),
            created_at: r.get("created_at"),
        }).collect();
        Ok((out, total))
    }

    pub async fn delete_customer(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM customers WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    // =================== INTERACTIONS ===================

    pub async fn put_interaction(&self, i: &Interaction) -> AppResult<Interaction> {
        let tags_json = serde_json::to_value(&i.tags)?;
        sqlx::query(
            "INSERT INTO interactions (id, agent_id, customer_id, channel, subject, transcript, tags, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (id) DO UPDATE SET agent_id=EXCLUDED.agent_id, customer_id=EXCLUDED.customer_id, channel=EXCLUDED.channel, subject=EXCLUDED.subject, transcript=EXCLUDED.transcript, tags=EXCLUDED.tags, updated_at=EXCLUDED.updated_at"
        )
        .bind(&i.id).bind(&i.agent_id).bind(&i.customer_id).bind(&i.channel).bind(&i.subject)
        .bind(&i.transcript).bind(tags_json).bind(i.created_at).bind(i.updated_at)
        .execute(&self.pool).await?;
        Ok(i.clone())
    }

    pub async fn get_interaction(&self, id: &str) -> AppResult<Option<Interaction>> {
        let row = sqlx::query("SELECT id, agent_id, customer_id, channel, subject, transcript, tags, created_at, updated_at FROM interactions WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        if let Some(r) = row {
            let tags: Vec<String> = serde_json::from_value(r.get("tags"))?;
            Ok(Some(Interaction {
                id: r.get("id"),
                agent_id: r.get("agent_id"),
                customer_id: r.get("customer_id"),
                channel: r.get("channel"),
                subject: r.get("subject"),
                transcript: r.get("transcript"),
                tags,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }))
        } else { Ok(None) }
    }

    pub async fn list_interactions(&self) -> AppResult<Vec<Interaction>> {
        let (items, _total) = self.list_interactions_paginated(1000, 0).await?;
        Ok(items)
    }

    pub async fn list_interactions_paginated(&self, limit: i64, offset: i64) -> AppResult<(Vec<Interaction>, i64)> {
        let count_row = sqlx::query("SELECT COUNT(*) AS cnt FROM interactions")
            .fetch_one(&self.pool).await?;
        let total: i64 = count_row.get("cnt");

        let rows = sqlx::query("SELECT id, agent_id, customer_id, channel, subject, transcript, tags, created_at, updated_at FROM interactions ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool).await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let tags: Vec<String> = serde_json::from_value(r.get("tags"))?;
            out.push(Interaction {
                id: r.get("id"),
                agent_id: r.get("agent_id"),
                customer_id: r.get("customer_id"),
                channel: r.get("channel"),
                subject: r.get("subject"),
                transcript: r.get("transcript"),
                tags,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            });
        }
        Ok((out, total))
    }

    // =================== RUBRICS ===================

    pub async fn put_rubric(&self, r: &Rubric) -> AppResult<()> {
        let crit_json = serde_json::to_value(&r.criteria)?;
        sqlx::query(
            "INSERT INTO rubrics (id, name, department, product_type, channel, version, criteria, active, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, department=EXCLUDED.department, product_type=EXCLUDED.product_type, channel=EXCLUDED.channel, version=EXCLUDED.version, criteria=EXCLUDED.criteria, active=EXCLUDED.active"
        )
        .bind(&r.id).bind(&r.name).bind(&r.department).bind(&r.product_type).bind(&r.channel)
        .bind(r.version as i32).bind(crit_json).bind(r.active).bind(r.created_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_rubric(&self, id: &str) -> AppResult<Option<Rubric>> {
        let row = sqlx::query("SELECT id, name, department, product_type, channel, version, criteria, active, created_at FROM rubrics WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        if let Some(r) = row {
            let criteria: Vec<RubricCriterion> = serde_json::from_value(r.get("criteria"))?;
            Ok(Some(Rubric {
                id: r.get("id"),
                name: r.get("name"),
                department: r.get("department"),
                product_type: r.get("product_type"),
                channel: r.get("channel"),
                version: r.get::<i32, _>("version") as u32,
                criteria,
                active: r.get("active"),
                created_at: r.get("created_at"),
            }))
        } else { Ok(None) }
    }

    pub async fn list_rubrics(&self) -> AppResult<Vec<Rubric>> {
        let rows = sqlx::query("SELECT id, name, department, product_type, channel, version, criteria, active, created_at FROM rubrics ORDER BY created_at DESC")
            .fetch_all(&self.pool).await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let criteria: Vec<RubricCriterion> = serde_json::from_value(r.get("criteria"))?;
            out.push(Rubric {
                id: r.get("id"),
                name: r.get("name"),
                department: r.get("department"),
                product_type: r.get("product_type"),
                channel: r.get("channel"),
                version: r.get::<i32, _>("version") as u32,
                criteria,
                active: r.get("active"),
                created_at: r.get("created_at"),
            });
        }
        Ok(out)
    }

    pub async fn ensure_default_rubric(&self) -> AppResult<()> {
        if !self.list_rubrics().await?.is_empty() {
            return Ok(());
        }
        self.ensure_default_metrics().await?;
        let metrics = self.list_metrics().await?;
        let by_code = |code: &str| -> String {
            metrics.iter().find(|m| m.code == code).map(|m| m.id.clone()).unwrap_or_default()
        };
        let r = Rubric {
            id: Uuid::new_v4().to_string(),
            name: "استاندارد پایه بانک/بیمه".into(),
            department: "عمومی".into(),
            product_type: None,
            channel: None,
            version: 1,
            criteria: vec![
                RubricCriterion { metric_id: by_code("compliance_identity_verification"), weight: 20.0, critical: true },
                RubricCriterion { metric_id: by_code("compliance_disclosure"),           weight: 20.0, critical: true },
                RubricCriterion { metric_id: by_code("communication_greeting"),          weight: 10.0, critical: false },
                RubricCriterion { metric_id: by_code("communication_active_listening"),  weight: 10.0, critical: false },
                RubricCriterion { metric_id: by_code("communication_empathy"),            weight: 10.0, critical: false },
                RubricCriterion { metric_id: by_code("resolution_first_call"),           weight: 20.0, critical: false },
                RubricCriterion { metric_id: by_code("resolution_followup_commitment"),  weight: 10.0, critical: false },
            ],
            active: true,
            created_at: Utc::now(),
        };
        self.put_rubric(&r).await
    }

    // =================== SCORES ===================

    pub async fn put_score(&self, s: &Score) -> AppResult<Score> {
        let dim_json = serde_json::to_value(&s.dimension_scores)?;
        let reasons_json = serde_json::to_value(&s.critical_fail_reasons)?;
        sqlx::query(
            "INSERT INTO scores (id, interaction_id, rubric_id, overall_score, level, dimension_scores, critical_fail, critical_fail_reasons, evaluator, notes, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT (interaction_id) DO UPDATE SET id=EXCLUDED.id, rubric_id=EXCLUDED.rubric_id, overall_score=EXCLUDED.overall_score, level=EXCLUDED.level, dimension_scores=EXCLUDED.dimension_scores, critical_fail=EXCLUDED.critical_fail, critical_fail_reasons=EXCLUDED.critical_fail_reasons, evaluator=EXCLUDED.evaluator, notes=EXCLUDED.notes"
        )
        .bind(&s.id).bind(&s.interaction_id).bind(&s.rubric_id).bind(s.overall_score).bind(&s.level)
        .bind(dim_json).bind(s.critical_fail).bind(reasons_json).bind(&s.evaluator).bind(&s.notes).bind(s.created_at)
        .execute(&self.pool).await?;
        Ok(s.clone())
    }

    pub async fn get_score_by_interaction(&self, interaction_id: &str) -> AppResult<Option<Score>> {
        let row = sqlx::query("SELECT id, interaction_id, rubric_id, overall_score, level, dimension_scores, critical_fail, critical_fail_reasons, evaluator, notes, created_at FROM scores WHERE interaction_id = $1")
            .bind(interaction_id).fetch_optional(&self.pool).await?;
        if let Some(r) = row {
            let dims: Vec<f64> = serde_json::from_value(r.get("dimension_scores"))?;
            let reasons: Vec<String> = serde_json::from_value(r.get("critical_fail_reasons"))?;
            Ok(Some(Score {
                id: r.get("id"),
                interaction_id: r.get("interaction_id"),
                rubric_id: r.get("rubric_id"),
                overall_score: r.get("overall_score"),
                level: r.get("level"),
                dimension_scores: dims,
                critical_fail: r.get("critical_fail"),
                critical_fail_reasons: reasons,
                evaluator: r.get("evaluator"),
                notes: r.get("notes"),
                created_at: r.get("created_at"),
            }))
        } else { Ok(None) }
    }

    pub async fn list_scores(&self) -> AppResult<Vec<Score>> {
        self.scan_scores().await
    }

    pub async fn scan_scores(&self) -> AppResult<Vec<Score>> {
        let rows = sqlx::query("SELECT id, interaction_id, rubric_id, overall_score, level, dimension_scores, critical_fail, critical_fail_reasons, evaluator, notes, created_at FROM scores")
            .fetch_all(&self.pool).await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let dims: Vec<f64> = serde_json::from_value(r.get("dimension_scores"))?;
            let reasons: Vec<String> = serde_json::from_value(r.get("critical_fail_reasons"))?;
            out.push(Score {
                id: r.get("id"),
                interaction_id: r.get("interaction_id"),
                rubric_id: r.get("rubric_id"),
                overall_score: r.get("overall_score"),
                level: r.get("level"),
                dimension_scores: dims,
                critical_fail: r.get("critical_fail"),
                critical_fail_reasons: reasons,
                evaluator: r.get("evaluator"),
                notes: r.get("notes"),
                created_at: r.get("created_at"),
            });
        }
        Ok(out)
    }

    // =================== ISSUES ===================

    pub async fn put_issue(&self, x: &Issue) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO issues (id, interaction_id, agent_id, severity, category, description, status, root_cause, corrective_action, due_at, created_at, resolved_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT (id) DO UPDATE SET interaction_id=EXCLUDED.interaction_id, agent_id=EXCLUDED.agent_id, severity=EXCLUDED.severity, category=EXCLUDED.category, description=EXCLUDED.description, status=EXCLUDED.status, root_cause=EXCLUDED.root_cause, corrective_action=EXCLUDED.corrective_action, due_at=EXCLUDED.due_at, resolved_at=EXCLUDED.resolved_at"
        )
        .bind(&x.id).bind(&x.interaction_id).bind(&x.agent_id).bind(&x.severity).bind(&x.category)
        .bind(&x.description).bind(&x.status).bind(&x.root_cause).bind(&x.corrective_action)
        .bind(x.due_at).bind(x.created_at).bind(x.resolved_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_issue(&self, id: &str) -> AppResult<Option<Issue>> {
        let row = sqlx::query("SELECT id, interaction_id, agent_id, severity, category, description, status, root_cause, corrective_action, due_at, created_at, resolved_at FROM issues WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| Issue {
            id: r.get("id"),
            interaction_id: r.get("interaction_id"),
            agent_id: r.get("agent_id"),
            severity: r.get("severity"),
            category: r.get("category"),
            description: r.get("description"),
            status: r.get("status"),
            root_cause: r.get("root_cause"),
            corrective_action: r.get("corrective_action"),
            due_at: r.get("due_at"),
            created_at: r.get("created_at"),
            resolved_at: r.get("resolved_at"),
        }))
    }

    pub async fn list_issues(&self) -> AppResult<Vec<Issue>> {
        let rows = sqlx::query("SELECT id, interaction_id, agent_id, severity, category, description, status, root_cause, corrective_action, due_at, created_at, resolved_at FROM issues ORDER BY created_at DESC")
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| Issue {
            id: r.get("id"),
            interaction_id: r.get("interaction_id"),
            agent_id: r.get("agent_id"),
            severity: r.get("severity"),
            category: r.get("category"),
            description: r.get("description"),
            status: r.get("status"),
            root_cause: r.get("root_cause"),
            corrective_action: r.get("corrective_action"),
            due_at: r.get("due_at"),
            created_at: r.get("created_at"),
            resolved_at: r.get("resolved_at"),
        }).collect())
    }

    pub async fn list_issues_paginated(&self, limit: i64, offset: i64, severity: Option<&str>, status: Option<&str>, agent_id: Option<&str>) -> AppResult<(Vec<Issue>, i64)> {
        // Build WHERE clause dynamically
        let mut where_clauses: Vec<String> = Vec::new();
        if severity.is_some() { where_clauses.push("severity = $3".to_string()); }
        if status.is_some() { where_clauses.push("status = $4".to_string()); }
        if agent_id.is_some() { where_clauses.push("agent_id = $5".to_string()); }
        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Count query
        let count_sql = format!("SELECT COUNT(*) FROM issues {}", where_sql);
        let mut count_query = sqlx::query_scalar(&count_sql);
        if let Some(s) = severity { count_query = count_query.bind(s); }
        if let Some(s) = status { count_query = count_query.bind(s); }
        if let Some(a) = agent_id { count_query = count_query.bind(a); }
        let total: i64 = count_query.fetch_one(&self.pool).await?;

        // List query
        let list_sql = format!("SELECT id, interaction_id, agent_id, severity, category, description, status, root_cause, corrective_action, due_at, created_at, resolved_at FROM issues {} ORDER BY created_at DESC LIMIT $1 OFFSET $2", where_sql);
        let mut list_query = sqlx::query(&list_sql).bind(limit).bind(offset);
        if let Some(s) = severity { list_query = list_query.bind(s); }
        if let Some(s) = status { list_query = list_query.bind(s); }
        if let Some(a) = agent_id { list_query = list_query.bind(a); }
        let rows = list_query.fetch_all(&self.pool).await?;
        let out: Vec<Issue> = rows.into_iter().map(|r| Issue {
            id: r.get("id"),
            interaction_id: r.get("interaction_id"),
            agent_id: r.get("agent_id"),
            severity: r.get("severity"),
            category: r.get("category"),
            description: r.get("description"),
            status: r.get("status"),
            root_cause: r.get("root_cause"),
            corrective_action: r.get("corrective_action"),
            due_at: r.get("due_at"),
            created_at: r.get("created_at"),
            resolved_at: r.get("resolved_at"),
        }).collect();
        Ok((out, total))
    }

    // =================== METRICS ===================

    pub async fn put_metric(&self, m: &MetricDefinition) -> AppResult<()> {
        let mt_str = serde_json::to_string(&m.metric_type)?.trim_matches('"').to_string();
        let av_json = serde_json::to_value(&m.allowed_values)?;
        let vs_json = serde_json::to_value(&m.value_scores)?;
        let rk_json = serde_json::to_value(&m.required_keywords)?;
        sqlx::query(
            "INSERT INTO metrics (id, code, title, description, category, metric_type, min_val, max_val, higher_is_better, allowed_values, value_scores, required_keywords, scale_min, scale_max, critical, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
             ON CONFLICT (code) DO UPDATE SET title=EXCLUDED.title, description=EXCLUDED.description, category=EXCLUDED.category, metric_type=EXCLUDED.metric_type, min_val=EXCLUDED.min_val, max_val=EXCLUDED.max_val, higher_is_better=EXCLUDED.higher_is_better, allowed_values=EXCLUDED.allowed_values, value_scores=EXCLUDED.value_scores, required_keywords=EXCLUDED.required_keywords, scale_min=EXCLUDED.scale_min, scale_max=EXCLUDED.scale_max, critical=EXCLUDED.critical"
        )
        .bind(&m.id).bind(&m.code).bind(&m.title).bind(&m.description).bind(&m.category).bind(&mt_str)
        .bind(m.min).bind(m.max).bind(m.higher_is_better)
        .bind(av_json).bind(vs_json).bind(rk_json)
        .bind(m.scale_min).bind(m.scale_max).bind(m.critical).bind(m.created_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_metric(&self, id: &str) -> AppResult<Option<MetricDefinition>> {
        let row = sqlx::query("SELECT * FROM metrics WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        if let Some(r) = row { Ok(Some(self.metric_from_row(&r)?)) } else { Ok(None) }
    }

    pub async fn get_metric_by_code(&self, code: &str) -> AppResult<Option<MetricDefinition>> {
        let row = sqlx::query("SELECT * FROM metrics WHERE code = $1")
            .bind(code).fetch_optional(&self.pool).await?;
        if let Some(r) = row { Ok(Some(self.metric_from_row(&r)?)) } else { Ok(None) }
    }

    pub async fn list_metrics(&self) -> AppResult<Vec<MetricDefinition>> {
        let rows = sqlx::query("SELECT * FROM metrics ORDER BY code")
            .fetch_all(&self.pool).await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows { out.push(self.metric_from_row(&r)?); }
        Ok(out)
    }

    fn metric_from_row(&self, r: &sqlx::postgres::PgRow) -> AppResult<MetricDefinition> {
        let mt_str: String = r.get("metric_type");
        let metric_type = match mt_str.as_str() {
            "boolean" => MetricType::Boolean,
            "numeric" => MetricType::Numeric,
            "categorical" => MetricType::Categorical,
            "text" => MetricType::Text,
            "scale" => MetricType::Scale,
            _ => MetricType::Boolean,
        };
        let av: Vec<String> = serde_json::from_value(r.get("allowed_values"))?;
        let vs: HashMap<String, f64> = serde_json::from_value(r.get("value_scores"))?;
        let rk: Vec<String> = serde_json::from_value(r.get("required_keywords"))?;
        Ok(MetricDefinition {
            id: r.get("id"),
            code: r.get("code"),
            title: r.get("title"),
            description: r.get("description"),
            category: r.get("category"),
            metric_type,
            min: r.get("min_val"),
            max: r.get("max_val"),
            higher_is_better: r.get("higher_is_better"),
            allowed_values: av,
            value_scores: vs,
            required_keywords: rk,
            scale_min: r.get("scale_min"),
            scale_max: r.get("scale_max"),
            critical: r.get("critical"),
            created_at: r.get("created_at"),
        })
    }

    pub async fn delete_metric(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM metrics WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn ensure_default_metrics(&self) -> AppResult<()> {
        if !self.list_metrics().await?.is_empty() { return Ok(()); }
        let defaults: Vec<MetricDefinition> = vec![
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "compliance_identity_verification".into(),
                title: "احراز هویت مشتری".into(),
                description: "آیا کارشناس قبل از ارائه خدمات، هویت مشتری را به درستی احراز کرده است؟".into(),
                category: "انطباق".into(),
                metric_type: MetricType::Boolean,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec![], value_scores: Default::default(),
                required_keywords: vec![], scale_min: None, scale_max: None,
                critical: true, created_at: Utc::now(),
            },
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "compliance_disclosure".into(),
                title: "ارائه اطلاعات الزامی".into(),
                description: "آیا اطلاعات الزامی (نرخ سود، کارمزد، شرایط) به مشتری اعلام شد؟".into(),
                category: "انطباق".into(),
                metric_type: MetricType::Text,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec![], value_scores: Default::default(),
                required_keywords: vec!["نرخ".into(), "کارمزد".into(), "شرایط".into()],
                scale_min: None, scale_max: None, critical: true, created_at: Utc::now(),
            },
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "communication_greeting".into(),
                title: "سلام و احوالپرسی".into(),
                description: "شروع مکالمه با سلام و احوالپرسی مناسب".into(),
                category: "ارتباط".into(),
                metric_type: MetricType::Scale,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec![], value_scores: Default::default(),
                required_keywords: vec![], scale_min: Some(1.0), scale_max: Some(5.0),
                critical: false, created_at: Utc::now(),
            },
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "communication_active_listening".into(),
                title: "گوش دادن فعال".into(),
                description: "تأیید درک صحیح صحبتهای مشتری".into(),
                category: "ارتباط".into(),
                metric_type: MetricType::Scale,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec![], value_scores: Default::default(),
                required_keywords: vec![], scale_min: Some(1.0), scale_max: Some(5.0),
                critical: false, created_at: Utc::now(),
            },
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "communication_empathy".into(),
                title: "همدلی با مشتری".into(),
                description: "ابراز درک شرایط مشتری".into(),
                category: "ارتباط".into(),
                metric_type: MetricType::Scale,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec![], value_scores: Default::default(),
                required_keywords: vec![], scale_min: Some(1.0), scale_max: Some(5.0),
                critical: false, created_at: Utc::now(),
            },
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "resolution_first_call".into(),
                title: "حل در تماس اول".into(),
                description: "آیا مشکل مشتری در همین تماس حل شد؟".into(),
                category: "حل مسئله".into(),
                metric_type: MetricType::Boolean,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec![], value_scores: Default::default(),
                required_keywords: vec![], scale_min: None, scale_max: None,
                critical: false, created_at: Utc::now(),
            },
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "resolution_followup_commitment".into(),
                title: "تعهد پیگیری".into(),
                description: "کارشناس متعهد به پیگیری شد؟".into(),
                category: "حل مسئله".into(),
                metric_type: MetricType::Boolean,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec![], value_scores: Default::default(),
                required_keywords: vec![], scale_min: None, scale_max: None,
                critical: false, created_at: Utc::now(),
            },
        ];
        for m in defaults { self.put_metric(&m).await?; }
        Ok(())
    }

    // =================== KPI ===================

    pub async fn put_kpi(&self, k: &Kpi) -> AppResult<()> {
        let kind_str = serde_json::to_string(&k.kind)?.trim_matches('"').to_string();
        sqlx::query(
            "INSERT INTO kpis (id, code, name, kind, description, pattern, threshold, ratio_total_pattern, weight, critical, active, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT (code) DO UPDATE SET name=EXCLUDED.name, kind=EXCLUDED.kind, description=EXCLUDED.description, pattern=EXCLUDED.pattern, threshold=EXCLUDED.threshold, ratio_total_pattern=EXCLUDED.ratio_total_pattern, weight=EXCLUDED.weight, critical=EXCLUDED.critical, active=EXCLUDED.active"
        )
        .bind(&k.id).bind(&k.code).bind(&k.name).bind(&kind_str).bind(&k.description)
        .bind(&k.pattern).bind(k.threshold).bind(&k.ratio_total_pattern)
        .bind(k.weight).bind(k.critical).bind(k.active).bind(k.created_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_kpi(&self, id: &str) -> AppResult<Option<Kpi>> {
        let row = sqlx::query("SELECT * FROM kpis WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        if let Some(r) = row { Ok(Some(self.kpi_from_row(&r)?)) } else { Ok(None) }
    }

    pub async fn get_kpi_by_code(&self, code: &str) -> AppResult<Option<Kpi>> {
        let row = sqlx::query("SELECT * FROM kpis WHERE code = $1")
            .bind(code).fetch_optional(&self.pool).await?;
        if let Some(r) = row { Ok(Some(self.kpi_from_row(&r)?)) } else { Ok(None) }
    }

    pub async fn list_kpis(&self) -> AppResult<Vec<Kpi>> {
        let rows = sqlx::query("SELECT * FROM kpis ORDER BY code")
            .fetch_all(&self.pool).await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows { out.push(self.kpi_from_row(&r)?); }
        Ok(out)
    }

    fn kpi_from_row(&self, r: &sqlx::postgres::PgRow) -> AppResult<Kpi> {
        let kind_str: String = r.get("kind");
        let kind = match kind_str.as_str() {
            "keyword_count" => KpiKind::KeywordCount,
            "keyword_presence" => KpiKind::KeywordPresence,
            "text_length" => KpiKind::TextLength,
            "keyword_ratio" => KpiKind::KeywordRatio,
            "response_time" => KpiKind::ResponseTime,
            "manual_range" => KpiKind::ManualRange,
            _ => KpiKind::KeywordPresence,
        };
        Ok(Kpi {
            id: r.get("id"),
            code: r.get("code"),
            name: r.get("name"),
            kind,
            description: r.get("description"),
            pattern: r.get("pattern"),
            threshold: r.get("threshold"),
            ratio_total_pattern: r.get("ratio_total_pattern"),
            weight: r.get("weight"),
            critical: r.get("critical"),
            active: r.get("active"),
            created_at: r.get("created_at"),
        })
    }

    pub async fn delete_kpi(&self, id: &str) -> AppResult<()> {
            sqlx::query("DELETE FROM kpis WHERE id = $1").bind(id).execute(&self.pool).await?;
            Ok(())
        }

        // =================== COACHING PLANS (Closed-Loop QA) ===================

        /// Create a new coaching plan in `draft` status.
        pub async fn create_coaching_plan(&self, p: &CoachingPlan) -> AppResult<()> {
            sqlx::query(
                "INSERT INTO coaching_plans
                    (id, agent_id, interaction_id, created_by, created_at,
                     coaching_theme, behavior_gap, evidence, root_cause, customer_impact,
                     practice_activity, success_metric, follow_up_due_at,
                     follow_up_review_count, status)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)"
            )
            .bind(&p.id)
            .bind(&p.agent_id)
            .bind(&p.interaction_id)
            .bind(&p.created_by)
            .bind(p.created_at)
            .bind(&p.coaching_theme)
            .bind(&p.behavior_gap)
            .bind(&p.evidence)
            .bind(&p.root_cause)
            .bind(&p.customer_impact)
            .bind(&p.practice_activity)
            .bind(&p.success_metric)
            .bind(p.follow_up_due_at)
            .bind(p.follow_up_review_count as i32)
            .bind(&p.status)
            .execute(&self.pool)
            .await?;
            Ok(())
        }

        pub async fn get_coaching_plan(&self, id: &str) -> AppResult<Option<CoachingPlan>> {
            let row = sqlx::query(
                "SELECT id, agent_id, interaction_id, created_by, created_at,
                        coaching_theme, behavior_gap, evidence, root_cause, customer_impact,
                        practice_activity, success_metric, follow_up_due_at,
                        follow_up_review_count, status,
                        acknowledged_at, acknowledged_note,
                        closed_at, closed_outcome, escalated_at
                 FROM coaching_plans WHERE id = $1"
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
            Ok(row.map(|r| CoachingPlan {
                id: r.get("id"),
                agent_id: r.get("agent_id"),
                interaction_id: r.get("interaction_id"),
                created_by: r.get("created_by"),
                created_at: r.get("created_at"),
                coaching_theme: r.get("coaching_theme"),
                behavior_gap: r.get("behavior_gap"),
                evidence: r.get("evidence"),
                root_cause: r.get("root_cause"),
                customer_impact: r.get("customer_impact"),
                practice_activity: r.get("practice_activity"),
                success_metric: r.get("success_metric"),
                follow_up_due_at: r.get("follow_up_due_at"),
                follow_up_review_count: r.get::<i32, _>("follow_up_review_count") as u32,
                status: r.get("status"),
                acknowledged_at: r.get("acknowledged_at"),
                acknowledged_note: r.get("acknowledged_note"),
                closed_at: r.get("closed_at"),
                closed_outcome: r.get("closed_outcome"),
                escalated_at: r.get("escalated_at"),
            }))
        }

        /// Partial update — only fields set to Some are written.
        pub async fn patch_coaching_plan(
            &self,
            id: &str,
            p: &CoachingPlanPatch,
        ) -> AppResult<()> {
            sqlx::query(
                "UPDATE coaching_plans SET
                    coaching_theme      = COALESCE($2, coaching_theme),
                    behavior_gap        = COALESCE($3, behavior_gap),
                    evidence            = COALESCE($4, evidence),
                    root_cause          = COALESCE($5, root_cause),
                    customer_impact     = COALESCE($6, customer_impact),
                    practice_activity   = COALESCE($7, practice_activity),
                    success_metric      = COALESCE($8, success_metric),
                    follow_up_due_at    = COALESCE($9, follow_up_due_at),
                    follow_up_review_count = COALESCE($10, follow_up_review_count),
                    status              = COALESCE($11, status)
                 WHERE id = $1"
            )
            .bind(id)
            .bind(&p.coaching_theme)
            .bind(&p.behavior_gap)
            .bind(&p.evidence)
            .bind(&p.root_cause)
            .bind(&p.customer_impact)
            .bind(&p.practice_activity)
            .bind(&p.success_metric)
            .bind(p.follow_up_due_at)
            .bind(p.follow_up_review_count.map(|n| n as i32))
            .bind(&p.status)
            .execute(&self.pool)
            .await?;
            Ok(())
        }

        /// Paginated list with optional filters. Mirrors `list_issues_paginated`.
        pub async fn list_coaching_plans(
            &self,
            agent_id: Option<&str>,
            status: Option<&str>,
            limit: i64,
            offset: i64,
        ) -> AppResult<(Vec<CoachingPlan>, i64)> {
            let mut where_sql = String::new();
            let mut idx = 1;
            if agent_id.is_some() { where_sql.push_str(&format!(" AND agent_id = ${idx}")); idx += 1; }
            if status.is_some() { where_sql.push_str(&format!(" AND status = ${idx}")); idx += 1; }
            let where_clause = if where_sql.is_empty() { String::new() } else { format!("WHERE 1=1{}", where_sql) };

            let count_sql = format!("SELECT COUNT(*) FROM coaching_plans {}", where_clause);
            let list_sql = format!(
                "SELECT id, agent_id, interaction_id, created_by, created_at,
                        coaching_theme, behavior_gap, evidence, root_cause, customer_impact,
                        practice_activity, success_metric, follow_up_due_at,
                        follow_up_review_count, status,
                        acknowledged_at, acknowledged_note,
                        closed_at, closed_outcome, escalated_at
                 FROM coaching_plans {} ORDER BY created_at DESC LIMIT ${idx} OFFSET ${}", where_clause, idx + 1);

            let total: i64 = {
                let mut cq = sqlx::query_scalar(&count_sql);
                if let Some(a) = agent_id { cq = cq.bind(a); }
                if let Some(s) = status { cq = cq.bind(s); }
                cq.fetch_one(&self.pool).await?
            };

            let mut q = sqlx::query(&list_sql);
            if let Some(a) = agent_id { q = q.bind(a); }
            if let Some(s) = status { q = q.bind(s); }
            q = q.bind(limit).bind(offset);

            let rows = q.fetch_all(&self.pool).await?;
            let plans = rows.into_iter().map(|r| CoachingPlan {
                id: r.get("id"),
                agent_id: r.get("agent_id"),
                interaction_id: r.get("interaction_id"),
                created_by: r.get("created_by"),
                created_at: r.get("created_at"),
                coaching_theme: r.get("coaching_theme"),
                behavior_gap: r.get("behavior_gap"),
                evidence: r.get("evidence"),
                root_cause: r.get("root_cause"),
                customer_impact: r.get("customer_impact"),
                practice_activity: r.get("practice_activity"),
                success_metric: r.get("success_metric"),
                follow_up_due_at: r.get("follow_up_due_at"),
                follow_up_review_count: r.get::<i32, _>("follow_up_review_count") as u32,
                status: r.get("status"),
                acknowledged_at: r.get("acknowledged_at"),
                acknowledged_note: r.get("acknowledged_note"),
                closed_at: r.get("closed_at"),
                closed_outcome: r.get("closed_outcome"),
                escalated_at: r.get("escalated_at"),
            }).collect();
            Ok((plans, total))
        }

        /// Apply a state-machine transition. Returns Err on invalid transition.
        /// This is the single source of truth for `coaching_plans.status` changes.
        pub async fn transition_coaching_plan(
            &self,
            id: &str,
            action: &str,
            outcome: Option<&str>,
            note: Option<&str>,
        ) -> AppResult<()> {
            let current = self
                .get_coaching_plan(id)
                .await?
                .ok_or_else(|| AppError::NotFound("coaching_plan".to_string()))?;
            let new_status = match (current.status.as_str(), action) {
                ("draft", "submit") => "pending_acknowledgement",
                ("pending_acknowledgement", "acknowledge") => "acknowledged",
                ("acknowledged" | "in_progress", "verify") => "verified",
                ("acknowledged" | "in_progress" | "verified" | "escalated", "close") => "closed",
                (_, "escalate") => "escalated",
                ("escalated", "resume") => "in_progress",
                ("escalated", "note") => "escalated",
                (s, a) => {
                    return Err(AppError::BadRequest(format!(
                        "invalid coaching plan transition '{a}' from '{s}'"
                    )));
                }
            };
            let q = match new_status {
                "pending_acknowledgement" =>
                    "UPDATE coaching_plans SET status=$1 WHERE id=$2".to_string(),
                "acknowledged" =>
                    "UPDATE coaching_plans SET status=$1, acknowledged_at=NOW(), acknowledged_note=$3 WHERE id=$2".to_string(),
                "verified" =>
                    "UPDATE coaching_plans SET status=$1 WHERE id=$2".to_string(),
                "closed" =>
                    "UPDATE coaching_plans SET status=$1, closed_at=NOW(), closed_outcome=$3 WHERE id=$2".to_string(),
                "escalated" =>
                    "UPDATE coaching_plans SET status=$1, escalated_at=NOW() WHERE id=$2".to_string(),
                "in_progress" =>
                    // resume from escalation: keep the note, clear escalation flag
                    "UPDATE coaching_plans SET status=$1, escalated_at=NULL WHERE id=$2".to_string(),
                _ => unreachable!(),
            };
            if action == "note" {
                // Manager note on an escalated plan: store it, keep status escalated.
                let note_q = "UPDATE coaching_plans SET acknowledged_note=$1, escalated_at=NOW() WHERE id=$2";
                sqlx::query(note_q)
                    .bind(note.unwrap_or(""))
                    .bind(id)
                    .execute(&self.pool)
                    .await?;
                return Ok(());
            }
            let mut exec = sqlx::query(&q).bind(new_status).bind(id);
            match new_status {
                "acknowledged" => {
                    exec = exec.bind(note);
                }
                "closed" => {
                    exec = exec.bind(outcome.unwrap_or("improved"));
                }
                _ => {}
            }
            exec.execute(&self.pool).await?;
            Ok(())
        }

        /// Record one follow-up measurement after a `submit_score`.
        /// If the count reaches the plan's `follow_up_review_count`, auto-transition
        /// the plan to `verified`.
        pub async fn record_coaching_follow_up(
            &self,
            fu: &CoachingFollowUp,
        ) -> AppResult<()> {
            let scores_json = serde_json::to_string(&fu.criterion_scores)
                .map_err(|e| AppError::Internal(format!("serialize criterion_scores: {e}")))?;
            sqlx::query(
                "INSERT INTO coaching_follow_ups
                    (id, plan_id, interaction_id, criterion_scores, overall_score, success)
                 VALUES ($1,$2,$3,$4::JSONB,$5,$6)"
            )
            .bind(&fu.id)
            .bind(&fu.plan_id)
            .bind(&fu.interaction_id)
            .bind(&scores_json)
            .bind(fu.overall_score)
            .bind(fu.success)
            .execute(&self.pool)
            .await?;

            // Count existing follow-ups for this plan.
            let count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM coaching_follow_ups WHERE plan_id = $1"
            )
            .bind(&fu.plan_id)
            .fetch_one(&self.pool)
            .await?;

            let plan = self
                .get_coaching_plan(&fu.plan_id)
                .await?
                .ok_or_else(|| AppError::NotFound("coaching_plan".to_string()))?;
            if count as u32 >= plan.follow_up_review_count
                && (plan.status == "acknowledged" || plan.status == "in_progress")
            {
                self.transition_coaching_plan(&fu.plan_id, "verify", None, None)
                    .await?;
            }
            Ok(())
        }

        pub async fn list_coaching_follow_ups(
            &self,
            plan_id: &str,
        ) -> AppResult<Vec<CoachingFollowUp>> {
            let rows = sqlx::query(
                "SELECT id, plan_id, interaction_id, measured_at,
                        criterion_scores, overall_score, success
                 FROM coaching_follow_ups WHERE plan_id = $1 ORDER BY measured_at"
            )
            .bind(plan_id)
            .fetch_all(&self.pool)
            .await?;
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                            let criterion_scores: std::collections::HashMap<String, f64> = r
                                .get::<serde_json::Value, _>("criterion_scores")
                                .as_object()
                                .map(|obj| {
                                    obj.iter()
                                        .filter_map(|(k, v)| v.as_f64().map(|n| (k.clone(), n)))
                                        .collect()
                                })
                                .unwrap_or_default();
                out.push(CoachingFollowUp {
                    id: r.get("id"),
                    plan_id: r.get("plan_id"),
                    interaction_id: r.get("interaction_id"),
                    measured_at: r.get("measured_at"),
                    criterion_scores,
                    overall_score: r.get("overall_score"),
                    success: r.get("success"),
                });
            }
            Ok(out)
        }

        /// Mark past-due plans in active states as `escalated`.
        /// Run on demand (cheap single SQL) — wired into the coaching summary endpoint.
        pub async fn escalate_overdue_coaching_plans(&self) -> AppResult<u64> {
            let res = sqlx::query(
                "UPDATE coaching_plans SET status='escalated', escalated_at=NOW()
                 WHERE status IN ('pending_acknowledgement','acknowledged','in_progress')
                   AND follow_up_due_at < NOW()
                   AND escalated_at IS NULL"
            )
            .execute(&self.pool)
            .await?;
            Ok(res.rows_affected())
        }

        /// Dashboard numbers. Calls `escalate_overdue_coaching_plans` first so the
        /// counts are accurate even without a cron.
        pub async fn get_coaching_summary(&self) -> AppResult<CoachingSummary> {
            let _ = self.escalate_overdue_coaching_plans().await?;
            let active: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM coaching_plans
                 WHERE status IN ('acknowledged','in_progress','verified')"
            ).fetch_one(&self.pool).await?;
            let pending_ack: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM coaching_plans WHERE status='pending_acknowledgement'"
            ).fetch_one(&self.pool).await?;
            let escalated: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM coaching_plans WHERE status='escalated'"
            ).fetch_one(&self.pool).await?;
            let avg_hours: Option<f64> = sqlx::query_scalar(
                            "SELECT AVG(EXTRACT(EPOCH FROM (acknowledged_at - created_at)) / 3600.0)::FLOAT8
                             FROM coaching_plans
                             WHERE acknowledged_at IS NOT NULL
                               AND created_at > NOW() - INTERVAL '30 days'"
                        ).fetch_one(&self.pool).await?;
                        let improvement_pct: Option<f64> = sqlx::query_scalar(
                            "SELECT
                               100.0 * SUM(CASE WHEN closed_outcome='improved' THEN 1 ELSE 0 END)::FLOAT8
                                    / NULLIF(COUNT(*) FILTER (WHERE status='closed'), 0)
                             FROM coaching_plans
                             WHERE closed_at > NOW() - INTERVAL '90 days'"
                        ).fetch_one(&self.pool).await?;
            Ok(CoachingSummary {
                active_count: active as u32,
                pending_ack_count: pending_ack as u32,
                escalated_count: escalated as u32,
                avg_time_to_ack_hours: avg_hours,
                improvement_rate_pct: improvement_pct,
            })
        }

        /// Look up the open plan for an agent + interaction (used by submit_score).
        pub async fn find_open_plan_for_interaction(
            &self,
            agent_id: &str,
            interaction_id: &str,
        ) -> AppResult<Option<CoachingPlan>> {
            let row = sqlx::query(
                "SELECT id, agent_id, interaction_id, created_by, created_at,
                        coaching_theme, behavior_gap, evidence, root_cause, customer_impact,
                        practice_activity, success_metric, follow_up_due_at,
                        follow_up_review_count, status,
                        acknowledged_at, acknowledged_note,
                        closed_at, closed_outcome, escalated_at
                 FROM coaching_plans
                 WHERE agent_id = $1 AND interaction_id = $2
                   AND status IN ('acknowledged','in_progress')
                 LIMIT 1"
            )
            .bind(agent_id)
            .bind(interaction_id)
            .fetch_optional(&self.pool)
            .await?;
            Ok(row.map(|r| CoachingPlan {
                id: r.get("id"),
                agent_id: r.get("agent_id"),
                interaction_id: r.get("interaction_id"),
                created_by: r.get("created_by"),
                created_at: r.get("created_at"),
                coaching_theme: r.get("coaching_theme"),
                behavior_gap: r.get("behavior_gap"),
                evidence: r.get("evidence"),
                root_cause: r.get("root_cause"),
                customer_impact: r.get("customer_impact"),
                practice_activity: r.get("practice_activity"),
                success_metric: r.get("success_metric"),
                follow_up_due_at: r.get("follow_up_due_at"),
                follow_up_review_count: r.get::<i32, _>("follow_up_review_count") as u32,
                status: r.get("status"),
                acknowledged_at: r.get("acknowledged_at"),
                acknowledged_note: r.get("acknowledged_note"),
                closed_at: r.get("closed_at"),
                closed_outcome: r.get("closed_outcome"),
                escalated_at: r.get("escalated_at"),
            }))
        }

        // =================== CALIBRATION SESSIONS ===================

        pub async fn create_calibration_session(&self, s: &CalibrationSession) -> AppResult<()> {
            let reviewer_json = serde_json::to_string(&s.reviewer_usernames)
                .map_err(|e| AppError::Internal(format!("serialize reviewer_usernames: {e}")))?;
            let sample_json = serde_json::to_string(&s.sample_interaction_ids)
                .map_err(|e| AppError::Internal(format!("serialize sample_interaction_ids: {e}")))?;
            sqlx::query(
                            "INSERT INTO calibration_sessions
                                                (id, name, status, rubric_id, reviewer_usernames, sample_interaction_ids,
                                                 target_agreement_rate, min_reviewers_per_interaction, deadline_at,
                                                 facilitator_id, created_at, meeting_started_at, agreement_rate, variance_per_criterion)
                                             VALUES ($1,$2,$3,$4,$5::JSONB,$6::JSONB,$7,$8,$9,$10,$11,$12,$13,$14)
                                             ON CONFLICT (id) DO UPDATE SET
                                                status = EXCLUDED.status,
                                                meeting_started_at = COALESCE(EXCLUDED.meeting_started_at, calibration_sessions.meeting_started_at),
                                                agreement_rate = COALESCE(EXCLUDED.agreement_rate, calibration_sessions.agreement_rate),
                                                variance_per_criterion = COALESCE(EXCLUDED.variance_per_criterion, calibration_sessions.variance_per_criterion)"
                        )
                        .bind(&s.id)
                        .bind(&s.name)
                        .bind(&s.status)
                        .bind(&s.rubric_id)
                        .bind(reviewer_json.as_str())
                        .bind(sample_json.as_str())
                        .bind(s.target_agreement_rate)
                        .bind(s.min_reviewers_per_interaction as i32)
                        .bind(&s.deadline_at)
                        .bind(&s.facilitator_id)
                        .bind(&s.created_at)
                        .bind(&s.meeting_started_at)
                        .bind(s.agreement_rate)
                        .bind(&s.variance_per_criterion)
                        .execute(&self.pool).await?;
                        Ok(())
        }

        pub async fn get_calibration_session(&self, id: &str) -> AppResult<Option<CalibrationSession>> {
            let row = sqlx::query(
                "SELECT * FROM calibration_sessions WHERE id = $1"
            )
            .bind(id)
            .fetch_optional(&self.pool).await?;
            Ok(row.map(|r| CalibrationSession {
                            id: r.get("id"),
                            name: r.get("name"),
                            status: r.get("status"),
                            rubric_id: r.get("rubric_id"),
                            reviewer_usernames: r.get::<serde_json::Value, _>("reviewer_usernames")
                                .as_array()
                                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                                .unwrap_or_default(),
                            sample_interaction_ids: r.get::<serde_json::Value, _>("sample_interaction_ids")
                                .as_array()
                                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                                .unwrap_or_default(),
                            target_agreement_rate: r.get("target_agreement_rate"),
                            min_reviewers_per_interaction: r.get::<i32, _>("min_reviewers_per_interaction") as u32,
                            deadline_at: r.get("deadline_at"),
                            meeting_started_at: r.get("meeting_started_at"),
                            facilitator_id: r.get("facilitator_id"),
                            agreement_rate: r.get("agreement_rate"),
                            variance_per_criterion: r.get("variance_per_criterion"),
                created_at: r.get("created_at"),
            }))
        }

        pub async fn list_calibration_sessions(
                    &self,
                    status: Option<&str>,
                    rubric_id: Option<&str>,
                    limit: i64,
                    offset: i64,
                ) -> AppResult<(Vec<CalibrationSession>, i64)> {
                    let mut where_sql = String::new();
                    let mut params: Vec<String> = Vec::new();
                    let mut idx = 1i32;
                    if let Some(st) = status {
                        where_sql.push_str(&format!(" AND status = ${idx}"));
                        params.push(st.to_string());
                        idx += 1;
                    }
                    if let Some(rubric) = rubric_id {
                        where_sql.push_str(&format!(" AND rubric_id = ${idx}"));
                        params.push(rubric.to_string());
                        idx += 1;
                    }
                    // count query: only where params (LIMIT/OFFSET not included)
                                let count_sql = format!("SELECT COUNT(*) FROM calibration_sessions WHERE 1=1 {where_sql}");
                                let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
                    for p in &params {
                        count_q = count_q.bind(p);
                    }
                    let total: i64 = count_q.fetch_one(&self.pool).await?;
                    // main query: where params + LIMIT/OFFSET after them
                    let limit_idx = idx; // next parameter index for LIMIT
                    let offset_idx = idx + 1;
                    let query_str = format!(
                        "SELECT * FROM calibration_sessions WHERE 1=1 {where_sql}
                         ORDER BY created_at DESC LIMIT ${limit_idx} OFFSET ${offset_idx}"
                    );
                    let mut q = sqlx::query(&query_str);
                    for p in &params {
                        q = q.bind(p);
                    }
                    q = q.bind(limit).bind(offset);
                    let rows = q.fetch_all(&self.pool).await?;
                                        let sessions = rows.into_iter().map(|r| CalibrationSession {
                                    id: r.get("id"),
                                    name: r.get("name"),
                                    status: r.get("status"),
                                    rubric_id: r.get("rubric_id"),
                                    reviewer_usernames: r.get::<serde_json::Value, _>("reviewer_usernames")
                                        .as_array()
                                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                                        .unwrap_or_default(),
                                    sample_interaction_ids: r.get::<serde_json::Value, _>("sample_interaction_ids")
                                        .as_array()
                                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                                        .unwrap_or_default(),
                                    target_agreement_rate: r.get("target_agreement_rate"),
                                    min_reviewers_per_interaction: r.get::<i32, _>("min_reviewers_per_interaction") as u32,
                                    deadline_at: r.get("deadline_at"),
                meeting_started_at: r.get("meeting_started_at"),
                facilitator_id: r.get("facilitator_id"),
                agreement_rate: r.get("agreement_rate"),
                variance_per_criterion: r.get("variance_per_criterion"),
                created_at: r.get("created_at"),
            }).collect();
            Ok((sessions, total))
        }

        pub async fn submit_calibration_score(&self, s: &CalibrationScore) -> AppResult<()> {
            let scores_json = serde_json::to_string(&s.criterion_scores)
                .map_err(|e| AppError::Internal(format!("serialize criterion_scores: {e}")))?;
            sqlx::query(
                "INSERT INTO calibration_scores
                    (id, session_id, interaction_id, reviewer, submitted_at,
                     criterion_scores, overall_score, notes)
                 VALUES ($1,$2,$3,$4,$5,$6::JSONB,$7,$8)
                 ON CONFLICT (session_id, interaction_id, reviewer)
                 DO UPDATE SET criterion_scores = $6::JSONB, overall_score = $7, notes = $8"
            )
            .bind(&s.id)
            .bind(&s.session_id)
            .bind(&s.interaction_id)
            .bind(&s.reviewer)
            .bind(&s.submitted_at)
            .bind(scores_json.as_str())
            .bind(s.overall_score)
            .bind(&s.notes)
            .execute(&self.pool).await?;
            Ok(())
        }

        pub async fn get_my_submission_status(
                    &self,
                    session_id: &str,
                    reviewer: &str,
                ) -> AppResult<(u32, u32)> {
                    let submitted: i64 = sqlx::query_scalar(
                        "SELECT COUNT(DISTINCT interaction_id) FROM calibration_scores
                         WHERE session_id = $1 AND reviewer = $2"
                    )
                    .bind(session_id)
                    .bind(reviewer)
                    .fetch_one(&self.pool).await?;
                    // total interactions in the session's sample list (JSONB -> text[] via jsonb_array_elements_text)
                    let total: i64 = sqlx::query_scalar(
                                    "SELECT COUNT(*) FROM calibration_sessions s,
                                            jsonb_array_elements_text(s.sample_interaction_ids) t
                                     WHERE s.id = $1"
                                )
                    .bind(session_id)
                    .fetch_one(&self.pool).await?;
                    Ok((submitted as u32, total as u32))
                }

        pub async fn save_calibration_decisions(&self, d: &CalibrationDecision) -> AppResult<()> {
            let example = d.example_interaction_id.as_deref().unwrap_or("");
            sqlx::query(
                "INSERT INTO calibration_decisions
                    (id, session_id, criterion_id, agreed_interpretation,
                     example_interaction_id, rubric_edit_proposed, created_at)
                 VALUES ($1,$2,$3,$4,$5,$6,$7)
                 ON CONFLICT (session_id, criterion_id)
                 DO UPDATE SET agreed_interpretation = $4,
                               example_interaction_id = $5,
                               rubric_edit_proposed = $6"
            )
            .bind(&d.id)
            .bind(&d.session_id)
            .bind(&d.criterion_id)
            .bind(&d.agreed_interpretation)
            .bind(example)
            .bind(d.rubric_edit_proposed.as_deref().unwrap_or(""))
            .bind(&d.created_at)
            .execute(&self.pool).await?;
            Ok(())
        }

        pub async fn compute_calibration_summary(&self, session_id: &str) -> AppResult<(Option<f64>, Option<serde_json::Value>)> {
            let scores = sqlx::query(
                "SELECT interaction_id, criterion_scores, overall_score
                 FROM calibration_scores WHERE session_id = $1"
            )
            .bind(session_id)
            .fetch_all(&self.pool).await?;
            // Group by interaction_id
            let mut by_interaction: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
            for r in &scores {
                            let iid: String = r.get("interaction_id");
                            let raw = r.get::<serde_json::Value, _>("criterion_scores");
                            if raw.is_object() {
                                by_interaction.entry(iid).or_insert_with(Vec::new).push(raw);
                            }
                        }
            let mut variance_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            let mut total_criteria = 0u32;
            let mut agreements = 0u32;
            for (_iid, groups) in by_interaction {
                if groups.len() < 2 { continue; }
                // Collect all criterion IDs across all groups for this interaction
                let mut all_criterion_ids: Vec<String> = Vec::new();
                for g in &groups {
                    if let Some(obj) = g.as_object() {
                        all_criterion_ids.extend(obj.keys().cloned());
                    }
                }
                all_criterion_ids.sort();
                all_criterion_ids.dedup();
                for cid in &all_criterion_ids {
                    let values: Vec<f64> = groups.iter()
                        .filter_map(|g| g.as_object().and_then(|o| o.get(cid)).and_then(|v| v.as_f64()))
                        .collect();
                    if values.len() < 2 { continue; }
                    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let variance = max - min;
                    variance_map.insert(cid.clone(), serde_json::json!({
                        "max": (max * 10.0).round() / 10.0,
                        "min": (min * 10.0).round() / 10.0,
                        "variance": (variance * 10.0).round() / 10.0
                    }));
                    total_criteria += 1;
                    if variance.abs() < 0.01 { agreements += 1; }
                }
            }
            let agreement_rate = if total_criteria > 0 {
                Some(agreements as f64 / total_criteria as f64 * 100.0)
            } else { None };
            Ok((agreement_rate, Some(serde_json::json!(variance_map))))
        }

        pub async fn get_rubric_decision_history(
            &self,
            rubric_id: &str,
            limit: i64,
            offset: i64,
        ) -> AppResult<(Vec<CalibrationDecision>, i64)> {
            // Get all sessions for this rubric
            let session_ids: Vec<String> = sqlx::query_scalar(
                            "SELECT id FROM calibration_sessions WHERE rubric_id = $1
                             ORDER BY created_at DESC LIMIT $2::bigint OFFSET $3::bigint"
                        )
            .bind(rubric_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool).await?;
            if session_ids.is_empty() { return Ok((vec![], 0)); }
            let n = session_ids.len();
                        let placeholders: Vec<String> = (1..=n).map(|i| format!("${i}")).collect();
                        let placeholders_str = placeholders.join(",");
                        let limit_pos = n + 1;
                        let offset_pos = n + 2;
                        let query = format!(
                            "SELECT * FROM calibration_decisions WHERE session_id IN ({})
                             ORDER BY created_at DESC LIMIT ${limit_pos}::bigint OFFSET ${offset_pos}::bigint",
                            placeholders_str,
                            limit_pos = limit_pos,
                            offset_pos = offset_pos
                        );
            let mut q = sqlx::query(&query);
            for sid in &session_ids { q = q.bind(sid); }
            q = q.bind(limit).bind(offset);
            let rows = q.fetch_all(&self.pool).await?;
            let count_query = format!(
                                            "SELECT COUNT(*) FROM calibration_decisions WHERE session_id IN ({})",
                                            placeholders_str
                                        );
                        let mut count_q = sqlx::query_scalar(&count_query);
                        for sid in &session_ids { count_q = count_q.bind(sid); }
                        let total: i64 = count_q.fetch_one(&self.pool).await?;
            let decisions = rows.into_iter().map(|r| CalibrationDecision {
                id: r.get("id"),
                session_id: r.get("session_id"),
                criterion_id: r.get("criterion_id"),
                agreed_interpretation: r.get("agreed_interpretation"),
                example_interaction_id: r.get("example_interaction_id"),
                rubric_edit_proposed: r.get("rubric_edit_proposed"),
                created_at: r.get("created_at"),
            }).collect();
            Ok((decisions, total))
        }

        // =================== DEMO SEED ===================

    /// Direct SQL seed of pre-computed scores and issues for the demo
    /// dataset. Called by lib.rs after seed_demo_data() so the dashboard
    /// has data to chart. We bypass the KPI engine here because the demo
    /// values are hand-picked to make the dashboard look realistic.
    pub async fn seed_scores_and_issues(&self) -> AppResult<()> {
            if self.scan_scores().await?.len() >= 8 { return Ok(()); }

            // Get actual interaction IDs from DB (not hardcoded - they change each seed)
            let interactions = self.list_interactions().await?;
            if interactions.len() < 8 {
                return Ok(()); // not enough interactions to seed
            }
            // Take first 8 interactions (sorted by created_at desc, so newest first)
            let mut sorted = interactions.clone();
            sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            let target_ids: Vec<String> = sorted.iter().take(8).map(|i| i.id.clone()).collect();

            // Pre-computed score profile mapped to interaction index in target_ids
            let profiles: Vec<(f64, &str, bool, Vec<&str>)> = vec![
                (64.7, "نیازمند بهبود", false, vec![]),     // افزایش سقف
                (6.98, "ضعیف", true, vec!["شکایت مشتری"]), // شکایت تأخیر
                (52.3, "ضعیف", false, vec![]),            // وام
                (57.2, "ضعیف", false, vec![]),            // افتتاح حساب
                (49.4, "ضعیف", false, vec![]),            // گزارش
                (61.9, "نیازمند بهبود", false, vec![]),   // تمدید بیمه
                (62.9, "نیازمند بهبود", false, vec![]),   // تحویل مدارک
                (49.0, "ضعیف", false, vec![]),            // افزایش سهم
            ];

            let agent_of = |iid: &str| -> String {
                interactions.iter().find(|x| x.id == iid).map(|x| x.agent_id.clone()).unwrap_or_default()
            };

            let now = Utc::now();
            for (iid, (overall, level, critical, reasons)) in target_ids.iter().zip(profiles.iter()) {
                let dim_count = 7;
                let dim_scores: Vec<f64> = (0..dim_count).map(|i| {
                    let frac = (overall / 100.0).clamp(0.0, 1.0);
                    let base = frac * 100.0;
                    let variation = (i as f64 * 7.3).sin().abs() * 20.0 - 10.0;
                    (base + variation).clamp(0.0, 100.0)
                }).collect();
                let s = Score {
                    id: self.next_id("scores_id_seq").await?,
                    interaction_id: iid.clone(),
                    rubric_id: String::new(),
                    overall_score: *overall,
                    level: level.to_string(),
                    dimension_scores: dim_scores,
                    critical_fail: *critical,
                    critical_fail_reasons: reasons.iter().map(|s| s.to_string()).collect(),
                    evaluator: "demo_seed".into(),
                    notes: "امتیاز ثبت شده توسط داده دمو".into(),
                    created_at: now - chrono::Duration::hours(2),
                };
                self.put_score(&s).await?;

                // Auto-issue
                let (sev, cat, desc) = if *critical {
                    ("بحرانی", "انطباق/دقت",
                     format!("شکست بحرانی: {}", reasons.join("، ")))
                } else if *overall < 60.0 {
                    ("بالا", "کیفیت کلی",
                     format!("امتیاز کیفیت {} کمتر از حد هشدار ۶۰", overall))
                } else {
                    ("متوسط", "بهبود",
                     format!("تعامل نیازمند برنامه بهبود؛ امتیاز {}", overall))
                };
                let due_days = if sev == "بحرانی" { 1 } else { 3 };
                let issue = Issue {
                    id: self.next_id("issues_id_seq").await?,
                    interaction_id: iid.clone(),
                    agent_id: agent_of(iid),
                    severity: sev.to_string(),
                    category: cat.to_string(),
                    description: desc,
                    status: "باز".into(),
                    root_cause: None,
                    corrective_action: None,
                    due_at: Some(Utc::now() + chrono::Duration::days(due_days)),
                    created_at: now - chrono::Duration::hours(1),
                    resolved_at: None,
                };
                self.put_issue(&issue).await?;
            }

            // Add a couple of resolved issues for the demo (history)
            // Find an interaction that wasn't already scored (use 9th if available)
            let resolved_target = sorted.get(8).map(|i| i.id.clone()).unwrap_or_else(|| target_ids[0].clone());
            let resolved = vec![
                Issue {
                    id: self.next_id("issues_id_seq").await?,
                    interaction_id: resolved_target.clone(),
                    agent_id: agent_of(&resolved_target),
                    severity: "متوسط".into(),
                    category: "اطلاعرسانی".into(),
                    description: "اطلاعات کافی درباره شرایط بیمه ارائه نشد".into(),
                    status: "بسته".into(),
                    root_cause: Some("آموزش ناقص کارشناس".into()),
                    corrective_action: Some("برگزاری دوره آموزشی بیمه".into()),
                    due_at: Some(now - chrono::Duration::days(1)),
                    created_at: now - chrono::Duration::days(2),
                    resolved_at: Some(now - chrono::Duration::days(1)),
                },
                Issue {
                    id: self.next_id("issues_id_seq").await?,
                    interaction_id: target_ids.get(4).cloned().unwrap_or_else(|| resolved_target.clone()), // 5th scored
                    agent_id: agent_of(target_ids.get(4).map(|s| s.as_str()).unwrap_or("")),
                    severity: "پایین".into(),
                    category: "مستندسازی".into(),
                    description: "گزارش ماهانه فاقد جزئیات کافی بود".into(),
                    status: "در حال بررسی".into(),
                    root_cause: None,
                    corrective_action: None,
                due_at: Some(now + chrono::Duration::days(2)),
                created_at: now - chrono::Duration::hours(12),
                resolved_at: None,
            },
        ];
        for i in &resolved { self.put_issue(i).await?; }

        Ok(())
    }

    pub async fn seed_demo_data(&self) -> AppResult<()> {
        // Allow forcing a fresh seed (used by demo / dev). Otherwise skip
        // if any agents already exist, to avoid duplicating rows on restart.
        if !self.list_agents().await?.is_empty()
            && std::env::var("FORCE_SEED").ok().as_deref() != Some("1")
        {
            return Ok(());
        }

        // ======== Realistic demo data =========
        // Imagine: 3 days of operation, 4 agents, 12 customers, ~30
        // interactions spread over time, with varied quality. Some scored
        // automatically, some manually, some with critical fails.

        // =========== Agents ===========
        let a1 = self.put_agent(&Agent {
            id: self.next_id("agents_id_seq").await?,
            name: "علی رضایی".into(),
            department: "بانک".into(),
            position: "کارشناس ارشد".into(),
            active: true,
            created_at: Utc::now() - chrono::Duration::days(3),
        }).await?;
        let _a2 = self.put_agent(&Agent {
            id: self.next_id("agents_id_seq").await?,
            name: "مریم کریمی".into(),
            department: "بیمه".into(),
            position: "کارشناس".into(),
            active: true,
            created_at: Utc::now() - chrono::Duration::days(2),
        }).await?;
        let _a3 = self.put_agent(&Agent {
            id: self.next_id("agents_id_seq").await?,
            name: "حسین نوری".into(),
            department: "سرمایهگذاری".into(),
            position: "کارشناس".into(),
            active: true,
            created_at: Utc::now() - chrono::Duration::days(2),
        }).await?;
        let _a4 = self.put_agent(&Agent {
            id: self.next_id("agents_id_seq").await?,
            name: "زهرا موسوی".into(),
            department: "بانک".into(),
            position: "کارشناس".into(),
            active: true,
            created_at: Utc::now() - chrono::Duration::days(1),
        }).await?;

        // =========== Customers ===========
        let cust1 = self.put_customer(&Customer {
            id: self.next_id("customers_id_seq").await?,
            name: "احمد محمدی".into(),
            phone: "09121234567".into(),
            product_type: "بانک".into(),
            segment: "VIP".into(),
            notes: "مشتری قدیمی، حساس به زمان پاسخگویی".into(),
            created_at: Utc::now() - chrono::Duration::days(3),
        }).await?;
        let cust2 = self.put_customer(&Customer {
            id: self.next_id("customers_id_seq").await?,
            name: "زهرا حسینی".into(),
            phone: "09359876543".into(),
            product_type: "بیمه".into(),
            segment: "عادی".into(),
            notes: "".into(),
            created_at: Utc::now() - chrono::Duration::days(2),
        }).await?;
        let cust3 = self.put_customer(&Customer {
            id: self.next_id("customers_id_seq").await?,
            name: "محمود کریمی".into(),
            phone: "09187654321".into(),
            product_type: "سرمایهگذاری".into(),
            segment: "مهم".into(),
            notes: "سرمایهگذار بلندمدت، علاقهمند به گزارش ماهانه".into(),
            created_at: Utc::now() - chrono::Duration::days(2),
        }).await?;
        let cust4 = self.put_customer(&Customer {
            id: self.next_id("customers_id_seq").await?,
            name: "فاطمه احمدی".into(),
            phone: "09361112233".into(),
            product_type: "وام".into(),
            segment: "عادی".into(),
            notes: "درخواست وام مسکن، پرونده ناقص".into(),
            created_at: Utc::now() - chrono::Duration::days(1),
        }).await?;
        let cust5 = self.put_customer(&Customer {
            id: self.next_id("customers_id_seq").await?,
            name: "علی اکبری".into(),
            phone: "09195556677".into(),
            product_type: "بانک".into(),
            segment: "مهم".into(),
            notes: "صاحب کسبوکار، حساب حقوقی".into(),
            created_at: Utc::now() - chrono::Duration::days(2),
        }).await?;

        // =========== Interactions ===========
        // (id is filled at insert time via put_interaction so it gets
        // a sequence-based id; we only construct the rest of the row.)
        async fn make_int(
            store: &Store,
            agent_id: &str,
            customer_id: &str,
            channel: &str,
            subject: &str,
            transcript: &str,
            tags: Vec<&str>,
            days_ago: i64,
        ) -> AppResult<Interaction> {
            let now = Utc::now() - chrono::Duration::days(days_ago)
                - chrono::Duration::hours((days_ago % 7) as i64);
            let i = Interaction {
                id: store.next_id("interactions_id_seq").await?,
                agent_id: agent_id.into(),
                customer_id: customer_id.into(),
                channel: channel.into(),
                subject: subject.into(),
                transcript: transcript.into(),
                tags: tags.into_iter().map(String::from).collect(),
                created_at: now,
                updated_at: now,
            };
            store.put_interaction(&i).await?;
            Ok(i)
        }
        let _i1 = make_int(&self, &a1.id, &cust1.id, "تلفن",
            "درخواست افزایش سقف اعتبار",
            "سلام وقت بخیر. بله، مشتری گرامی. احراز هویت انجام شد. نرخ فعلی ۱۸٪ و کارمزد ماهانه ۵۰۰۰ تومان است. شرایط ویژه برای شما فعال شد. متشکریم از تماس شما. خداحافظ.",
            vec!["افزایش_سقف", "VIP"], 2).await?;
        let _i2 = make_int(&self, &a1.id, &cust2.id, "چت",
            "سوال درباره بیمه نامه",
            "سلام. منظور شما را متوجه شدم. متأسفانه شرایط فعلی اجازه نمیدهد. پیگیری میکنم و خبر میدهم.",
            vec!["بیمه"], 2).await?;
        let _i3 = make_int(&self, &a1.id, &cust1.id, "ایمیل",
            "شکایت از تأخیر در پاسخگویی",
            "با عرض پوزش بابت تأخیر. مشکل شما را بررسی کردم. این کار بسیار بد مایه تأسف است. قول میدهم سریعتر حل شود.",
            vec!["شکایت"], 1).await?;
        let _i4 = make_int(&self, &a1.id, &cust4.id, "تلفن",
            "پیگیری وام مسکن",
            "سلام. احراز هویت انجام شد. مدارک شما ناقص است. لطفاً فیش حقوقی و سند ملک را ارسال کنید. نرخ سود ۲۲٪ و شرایط بازپرداخت ۲۰ سال است.",
            vec!["وام", "ناقص"], 1).await?;
        let _i5 = make_int(&self, &a1.id, &cust5.id, "تلفن",
            "افتتاح حساب حقوقی",
            "سلام. بله. احراز هویت انجام شد. مدارک لازم را بفرستید. کارمزد ماهانه ۲۰۰۰۰ تومان. شرایط ویژه برای کسبوکار شما فعال میشود. متشکرم.",
            vec!["بانک", "حقوقی"], 0).await?;
        let _i6 = make_int(&self, &a1.id, &cust3.id, "ایمیل",
            "گزارش ماهانه سرمایهگذاری",
            "سلام. گزارش ماهانه شما آماده است. سود این ماه ۱۲٪ بود. متشکریم.",
            vec!["گزارش"], 0).await?;
        let _i7 = make_int(&self, &a1.id, &cust2.id, "تلفن",
            "تمدید بیمه",
            "سلام. بله. احراز هویت شد. نرخ ۸٪ و شرایط تمدید ۱ ساله. کارمزد ۵۰۰۰۰ تومان. خداحافظ.",
            vec!["بیمه", "تمدید"], 0).await?;
        let _i8 = make_int(&self, &a1.id, &cust1.id, "تلفن",
            "پیگیری وضعیت درخواست",
            "سلام. درخواست شما در حال بررسی است. پیگیری میکنم. خداحافظ.",
            vec!["پیگیری"], 0).await?;
        let _i9 = make_int(&self, &a1.id, &cust5.id, "چت",
            "مشکل با اپلیکیشن",
            "سلام. لطفاً نسخه اپ را بهروز کنید. متأسفانه مشکل شناخته شدهای است. پیگیری میشود.",
            vec!["فنی"], 0).await?;
        let _i10 = make_int(&self, &a1.id, &cust4.id, "تلفن",
            "تحویل مدارک",
            "سلام. مدارک را دریافت کردم. احراز هویت انجام شد. نرخ ۲۲٪ و شرایط بازپرداخت ۲۰ سال. منتظر تأیید نهایی باشید. خداحافظ.",
            vec!["وام", "تکمیل"], 0).await?;
        let _i11 = make_int(&self, &a1.id, &cust3.id, "تلفن",
            "افزایش سهم سرمایهگذاری",
            "سلام. درخواست شما ثبت شد. سود این ماه ۱۲٪. متشکریم.",
            vec!["سرمایهگذاری"], 0).await?;
        let _i12 = make_int(&self, &a1.id, &cust1.id, "ایمیل",
            "تقدیر و تشکر",
            "سلام. از بازخورد مثبت شما متشکریم. خداحافظ.",
            vec!["تقدیر"], 0).await?;

        Ok(())
            }

            /// Seed Persian coaching plans and calibration sessions for demo.
            /// Called after seed_scores_and_issues so interactions and agents exist.
            pub async fn seed_persian_coaching_calibration(&self) -> AppResult<()> {
                            // Skip if any coaching plans already exist (idempotent).
                            // NOTE: tables are seeded once; on a DB with stale pre-seed
                            // junk this guard skips, so demo tables were cleaned first.
                            let existing = self.list_coaching_plans(None, None, 1, 0).await?;
                            if !existing.0.is_empty() {
                                return Ok(());
                            }

                let agents = self.list_agents().await?;
                let interactions = self.list_interactions().await?;
                if agents.is_empty() || interactions.is_empty() {
                    return Ok(());
                }

                // Use the first few agents and interactions
                let agent_ids: Vec<String> = agents.iter().take(3).map(|a| a.id.clone()).collect();
                let int_ids: Vec<String> = interactions.iter().take(6).map(|i| i.id.clone()).collect();
                let now = Utc::now();

                // ---------- Coaching Plans (Persian) ----------
                // Plan 1: draft → will be submitted
                let cp1 = CoachingPlan {
                    id: self.next_id("coaching_plans_id_seq").await?,
                    agent_id: agent_ids[0].clone(),
                    interaction_id: int_ids[0].clone(),
                    created_by: "demo_admin".into(),
                    created_at: now - chrono::Duration::days(2),
                    coaching_theme: "احوالپرسی و بازخورد مثبت".into(),
                    behavior_gap: "کارشناس در شروع مکالمه احوالپرسی نمی‌کند و مستقیماً موضوع را آغاز می‌کند".into(),
                    evidence: format!("مکالمه {} — первые ۳۰ ثانیه فاقد احوالپرسی", int_ids[0]),
                    root_cause: "عدم تمرین اسکریپت ورود به مکالمه".into(),
                    customer_impact: "مشتری حس می‌کند اهمیتی ندارد و اعتماد اولیه پایین می‌آید".into(),
                    practice_activity: "نقش‌بازی (roleplay) ۱۰ دقیقه‌ای با مربی".into(),
                    success_metric: "میانگین امتیاز احوالپرسی ≥ ۸۰".into(),
                    follow_up_due_at: now + chrono::Duration::days(7),
                    follow_up_review_count: 2,
                    status: "draft".into(),
                    acknowledged_at: None,
                    acknowledged_note: None,
                    closed_at: None,
                    closed_outcome: None,
                    escalated_at: None,
                };
                self.create_coaching_plan(&cp1).await?;

                // Plan 2: pending_acknowledgement
                let cp2 = CoachingPlan {
                    id: self.next_id("coaching_plans_id_seq").await?,
                    agent_id: agent_ids[1].clone(),
                    interaction_id: int_ids[1].clone(),
                    created_by: "demo_admin".into(),
                    created_at: now - chrono::Duration::days(1),
                    coaching_theme: "مدیریت اعتراض مشتری".into(),
                    behavior_gap: "کارشناس در مواجهه با اعتراض، دفاعی می‌شود و اصرار دارد تا پذیرش اعتراض".into(),
                    evidence: format!("مکالمه {} — دقیقه ۲ تا ۴", int_ids[1]),
                    root_cause: "کمبود مهارت گوش دادن فعال و احساسات‌مدیریت".into(),
                    customer_impact: "اعتراض تشدید شده و مشتری تهدید به شکایت کرده است".into(),
                    practice_activity: "مشاهده ویدیوی نمونه مدیریت اعتراض + تمرین".into(),
                    success_metric: "نرخ حل اعتراض در تماس اول ≥ ۷۰٪".into(),
                    follow_up_due_at: now + chrono::Duration::days(5),
                    follow_up_review_count: 3,
                    status: "pending_acknowledgement".into(),
                    acknowledged_at: None,
                    acknowledged_note: None,
                    closed_at: None,
                    closed_outcome: None,
                    escalated_at: None,
                };
                self.create_coaching_plan(&cp2).await?;

                // Plan 3: acknowledged (in progress)
                let cp3 = CoachingPlan {
                    id: self.next_id("coaching_plans_id_seq").await?,
                    agent_id: agent_ids[0].clone(),
                    interaction_id: int_ids[2].clone(),
                    created_by: "demo_admin".into(),
                    created_at: now - chrono::Duration::days(3),
                    coaching_theme: "کامل کردن پرونده و مستندسازی".into(),
                    behavior_gap: "اطلاعات ضروری مشتری (کد ملی، شماره تماس، آدرس) در سیستم ثبت نشده است".into(),
                    evidence: format!("مکالمه {} — پرونده ناقص", int_ids[2]),
                    root_cause: "بی‌توجهی به چک‌لیست ورود اطلاعات".into(),
                    customer_impact: "نیاز به تماس مجدد برای تکمیل اطلاعات، افزایش زمان پردازش".into(),
                    practice_activity: "بررسی چک‌لیست + تکمیل پرونده ۵ نمونه".into(),
                    success_metric: "نرخ پرونده‌های کامل در اولین بار ≥ ۹۰٪".into(),
                    follow_up_due_at: now + chrono::Duration::days(10),
                    follow_up_review_count: 2,
                    status: "acknowledged".into(),
                    acknowledged_at: Some(now - chrono::Duration::days(1)),
                    acknowledged_note: Some("برنامه تایید و آغاز شد".into()),
                    closed_at: None,
                    closed_outcome: None,
                    escalated_at: None,
                };
                self.create_coaching_plan(&cp3).await?;

                // Plan 4: closed (successful)
                let cp4 = CoachingPlan {
                    id: self.next_id("coaching_plans_id_seq").await?,
                    agent_id: agent_ids[2].clone(),
                    interaction_id: int_ids[3].clone(),
                    created_by: "demo_admin".into(),
                    created_at: now - chrono::Duration::days(5),
                    coaching_theme: "فروش متقاطع (Cross-sell) مناسب".into(),
                    behavior_gap: "کارشناس محصولات مکمل را بر اساس نیاز مشتری پیشنهاد نمی‌دهد".into(),
                    evidence: format!("مکالمه {} — فرصت فروش از دست رفته", int_ids[3]),
                    root_cause: "نشناختن سیگنال‌های نیاز مشتری".into(),
                    customer_impact: "از دست رفتن درآمد و کاهش سهم کیف پول مشتری".into(),
                    practice_activity: "آموزش تشخیص سیگنال + نقش‌بازی فروش متقاطع".into(),
                    success_metric: "نرخ پیشنهاد محصول مکمل ≥ ۶۰٪".into(),
                    follow_up_due_at: now - chrono::Duration::days(2),
                    follow_up_review_count: 2,
                    status: "closed".into(),
                    acknowledged_at: Some(now - chrono::Duration::days(4)),
                    acknowledged_note: Some("شروع شد".into()),
                    closed_at: Some(now - chrono::Duration::days(1)),
                    closed_outcome: Some("improved".into()),
                    escalated_at: None,
                };
                self.create_coaching_plan(&cp4).await?;

                // Plan 5: escalated (overdue)
                let cp5 = CoachingPlan {
                    id: self.next_id("coaching_plans_id_seq").await?,
                    agent_id: agent_ids[1].clone(),
                    interaction_id: int_ids[4].clone(),
                    created_by: "demo_admin".into(),
                    created_at: now - chrono::Duration::days(10),
                    coaching_theme: "تأیید هویت و امنیت".into(),
                    behavior_gap: "مراحل تأیید هویت (شامل پرسش امنیتی) انجام نشده است".into(),
                    evidence: format!("مکالمه {} — عدم تأیید هویت کامل", int_ids[4]),
                    root_cause: "آشنایی ناکافی با پروتکل امنیتی".into(),
                    customer_impact: "ریسک امنیتی و نقض مقررات بانک مرکزی".into(),
                    practice_activity: "مرور پروتکل امنیتی + تست عملی".into(),
                    success_metric: "نرخ تأیید هویت کامل ۱۰۰٪".into(),
                    follow_up_due_at: now - chrono::Duration::days(5),
                    follow_up_review_count: 2,
                    status: "escalated".into(),
                    acknowledged_at: Some(now - chrono::Duration::days(8)),
                    acknowledged_note: Some("ارجاع شد".into()),
                    closed_at: None,
                    closed_outcome: None,
                    escalated_at: Some(now - chrono::Duration::days(1)),
                };
                self.create_coaching_plan(&cp5).await?;

                // ---------- Calibration Sessions (Persian) ----------
                // Session 1: completed (with agreement rate)
                let rubrics = self.list_rubrics().await?;
                        let rubric_id = rubrics.first().map(|r| r.id.clone()).unwrap_or("1".into());

                        // Session 1: completed (with agreement rate)
                        let cs1 = CalibrationSession {
                            id: self.next_id("calibration_sessions_id_seq").await?,
                            name: "کالیبراسیون مهرماه — بخش بانک".into(),
                            status: "completed".into(),
                            rubric_id: rubric_id.clone(),
                    reviewer_usernames: vec!["demo_admin".into(), "مریم کریمی".into(), "حسین نوری".into()],
                    sample_interaction_ids: int_ids[0..3].to_vec(),
                    target_agreement_rate: Some(0.8),
                    min_reviewers_per_interaction: 2,
                    deadline_at: now + chrono::Duration::days(30),
                    meeting_started_at: Some(now - chrono::Duration::days(2)),
                    facilitator_id: Some("demo_admin".into()),
                    agreement_rate: Some(78.5),
                    variance_per_criterion: Some(serde_json::json!({
                        "احوالپرسی": {"max": 5.0, "min": 5.0, "variance": 0.0},
                        "دقت اطلاعات": {"max": 3.0, "min": 3.0, "variance": 0.0},
                        "حل مسئله": {"max": 4.0, "min": 2.0, "variance": 2.0}
                    })),
                    created_at: now - chrono::Duration::days(10),
                };
                self.create_calibration_session(&cs1).await?;

                // Session 2: in_session (scoring phase)
                let cs2 = CalibrationSession {
                    id: self.next_id("calibration_sessions_id_seq").await?,
                    name: "کالیبراسیون آبان ماه — بخش بیمه".into(),
                    status: "in_session".into(),
                    rubric_id: rubric_id.clone(),
                    reviewer_usernames: vec!["demo_admin".into(), "زهرا موسوی".into()],
                    sample_interaction_ids: int_ids[3..5].to_vec(),
                    target_agreement_rate: Some(0.75),
                    min_reviewers_per_interaction: 2,
                    deadline_at: now + chrono::Duration::days(15),
                    meeting_started_at: Some(now - chrono::Duration::hours(2)),
                    facilitator_id: Some("demo_admin".into()),
                    agreement_rate: None,
                    variance_per_criterion: None,
                    created_at: now - chrono::Duration::days(3),
                };
                self.create_calibration_session(&cs2).await?;

                // Session 3: draft (ready to start)
                let cs3 = CalibrationSession {
                    id: self.next_id("calibration_sessions_id_seq").await?,
                    name: "کالیبراسیون آذرماه — بخش سرمایه‌گذاری".into(),
                    status: "draft".into(),
                    rubric_id: rubric_id.clone(),
                    reviewer_usernames: vec!["demo_admin".into(), "علی رضایی".into(), "مریم کریمی".into()],
                    sample_interaction_ids: int_ids[4..6].to_vec(),
                    target_agreement_rate: Some(0.85),
                    min_reviewers_per_interaction: 2,
                    deadline_at: now + chrono::Duration::days(45),
                    meeting_started_at: None,
                    facilitator_id: Some("demo_admin".into()),
                    agreement_rate: None,
                    variance_per_criterion: None,
                    created_at: now,
                };
                self.create_calibration_session(&cs3).await?;

                Ok(())
            }
        }
