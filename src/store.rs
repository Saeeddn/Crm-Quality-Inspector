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
        ];
        for sql in stmts {
            sqlx::query(sql).execute(&self.pool).await?;
        }
        Ok(())
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
        if self.user_exists(username).await? {
            return Ok(());
        }
        let hash = crate::auth::hash_password(password)?;
        let user = User {
            username: username.into(),
            password_hash: hash,
            is_admin: true,
            created_at: Utc::now(),
        };
        self.put_user(&user).await
    }

    // =================== AGENTS ===================

    pub async fn put_agent(&self, a: &Agent) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO agents (id, name, department, position, active, created_at) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, department=EXCLUDED.department, position=EXCLUDED.position, active=EXCLUDED.active"
        )
        .bind(&a.id).bind(&a.name).bind(&a.department).bind(&a.position)
        .bind(a.active).bind(a.created_at)
        .execute(&self.pool).await?;
        Ok(())
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

    pub async fn delete_agent(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM agents WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    // =================== CUSTOMERS ===================

    pub async fn put_customer(&self, x: &Customer) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO customers (id, name, phone, product_type, segment, notes, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, phone=EXCLUDED.phone, product_type=EXCLUDED.product_type, segment=EXCLUDED.segment, notes=EXCLUDED.notes"
        )
        .bind(&x.id).bind(&x.name).bind(&x.phone).bind(&x.product_type).bind(&x.segment).bind(&x.notes).bind(x.created_at)
        .execute(&self.pool).await?;
        Ok(())
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

    pub async fn delete_customer(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM customers WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    // =================== INTERACTIONS ===================

    pub async fn put_interaction(&self, i: &Interaction) -> AppResult<()> {
        let tags_json = serde_json::to_value(&i.tags)?;
        sqlx::query(
            "INSERT INTO interactions (id, agent_id, customer_id, channel, subject, transcript, tags, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (id) DO UPDATE SET agent_id=EXCLUDED.agent_id, customer_id=EXCLUDED.customer_id, channel=EXCLUDED.channel, subject=EXCLUDED.subject, transcript=EXCLUDED.transcript, tags=EXCLUDED.tags, updated_at=EXCLUDED.updated_at"
        )
        .bind(&i.id).bind(&i.agent_id).bind(&i.customer_id).bind(&i.channel).bind(&i.subject)
        .bind(&i.transcript).bind(tags_json).bind(i.created_at).bind(i.updated_at)
        .execute(&self.pool).await?;
        Ok(())
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
        let rows = sqlx::query("SELECT id, agent_id, customer_id, channel, subject, transcript, tags, created_at, updated_at FROM interactions ORDER BY created_at DESC")
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
        Ok(out)
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

    pub async fn put_score(&self, s: &Score) -> AppResult<()> {
        let dim_json = serde_json::to_value(&s.dimension_scores)?;
        let reasons_json = serde_json::to_value(&s.critical_fail_reasons)?;
        sqlx::query(
            "INSERT INTO scores (id, interaction_id, rubric_id, overall_score, level, dimension_scores, critical_fail, critical_fail_reasons, evaluator, notes, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT (interaction_id) DO UPDATE SET id=EXCLUDED.id, rubric_id=EXCLUDED.rubric_id, overall_score=EXCLUDED.overall_score, level=EXCLUDED.level, dimension_scores=EXCLUDED.dimension_scores, critical_fail=EXCLUDED.critical_fail, critical_fail_reasons=EXCLUDED.critical_fail_reasons, evaluator=EXCLUDED.evaluator, notes=EXCLUDED.notes"
        )
        .bind(&s.id).bind(&s.interaction_id).bind(&s.rubric_id).bind(s.overall_score).bind(&s.level)
        .bind(dim_json).bind(s.critical_fail).bind(reasons_json).bind(&s.evaluator).bind(&s.notes).bind(s.created_at)
        .execute(&self.pool).await?;
        Ok(())
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

    // =================== DEMO SEED ===================

    pub async fn seed_demo_data(&self) -> AppResult<()> {
        if !self.list_agents().await?.is_empty() { return Ok(()); }

        let agent1 = Agent {
            id: Uuid::new_v4().to_string(),
            name: "علی رضایی".into(),
            department: "بانک".into(),
            position: "کارشناس ارشد".into(),
            active: true,
            created_at: Utc::now(),
        };
        let agent2 = Agent {
            id: Uuid::new_v4().to_string(),
            name: "مریم کریمی".into(),
            department: "بیمه".into(),
            position: "کارشناس".into(),
            active: true,
            created_at: Utc::now(),
        };
        self.put_agent(&agent1).await?;
        self.put_agent(&agent2).await?;

        let cust1 = Customer {
            id: Uuid::new_v4().to_string(),
            name: "احمد محمدی".into(),
            phone: "09121234567".into(),
            product_type: "بانک".into(),
            segment: "VIP".into(),
            notes: "مشتری قدیمی".into(),
            created_at: Utc::now(),
        };
        let cust2 = Customer {
            id: Uuid::new_v4().to_string(),
            name: "زهرا حسینی".into(),
            phone: "09359876543".into(),
            product_type: "بیمه".into(),
            segment: "عادی".into(),
            notes: "".into(),
            created_at: Utc::now(),
        };
        self.put_customer(&cust1).await?;
        self.put_customer(&cust2).await?;

        let i1 = Interaction {
            id: Uuid::new_v4().to_string(),
            agent_id: agent1.id.clone(),
            customer_id: cust1.id.clone(),
            channel: "تلفن".into(),
            subject: "درخواست افزایش سقف اعتبار".into(),
            transcript: "سلام. بله، مشتری گرامی. احراز هویت انجام شد. نرخ فعلی ۱۸٪ و کارمزد ماهانه ۵۰۰۰ تومان. شرایط ویژه برای شما فعال شد. متشکریم از تماس شما. خداحافظ.".into(),
            tags: vec!["افزایش_سقف".into(), "VIP".into()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let i2 = Interaction {
            id: Uuid::new_v4().to_string(),
            agent_id: agent2.id.clone(),
            customer_id: cust2.id.clone(),
            channel: "چت".into(),
            subject: "سوال درباره بیمه نامه".into(),
            transcript: "سلام. منظور شما را متوجه شدم. متأسفانه شرایط فعلی اجازه نمیدهد. پیگیری میکنم و خبر میدهم.".into(),
            tags: vec!["بیمه".into()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let i3 = Interaction {
            id: Uuid::new_v4().to_string(),
            agent_id: agent1.id.clone(),
            customer_id: cust2.id.clone(),
            channel: "ایمیل".into(),
            subject: "شکایت از تأخیر".into(),
            transcript: "با عرض پوزش بابت تأخیر. مشکل شما را بررسی کردم. این کار بسیار بد مایه تأسف است. قول میدهم سریعتر حل شود.".into(),
            tags: vec!["شکایت".into()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.put_interaction(&i1).await?;
        self.put_interaction(&i2).await?;
        self.put_interaction(&i3).await?;
        Ok(())
    }
}
