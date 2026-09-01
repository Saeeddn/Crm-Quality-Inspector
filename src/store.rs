use crate::error::{AppError, AppResult};
use crate::models::*;
use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

const PREFIX_USER: &str = "u:";
const PREFIX_AGENT: &str = "ag:";
const PREFIX_CUSTOMER: &str = "cu:";
const PREFIX_INTERACTION: &str = "in:";
const PREFIX_RUBRIC: &str = "rb:";
const PREFIX_SCORE: &str = "sc:";
const PREFIX_ISSUE: &str = "is:";
const PREFIX_METRIC: &str = "mt:";
const PREFIX_KPI: &str = "kp:";
const SET_USERS: &str = "set:users";
const SET_AGENTS: &str = "set:agents";
const SET_CUSTOMERS: &str = "set:customers";
const SET_INTERACTIONS: &str = "set:interactions";
const SET_RUBRICS: &str = "set:rubrics";
const SET_SCORES: &str = "set:scores";
const SET_ISSUES: &str = "set:issues";
const SET_METRICS: &str = "set:metrics";
const SET_METRIC_BY_CODE: &str = "idx:metric_by_code";
const SET_KPIS: &str = "set:kpis";
const SET_KPI_BY_CODE: &str = "idx:kpi_by_code";

#[derive(Clone)]
pub struct Store {
    client: redis::Client,
}

impl Store {
    pub async fn connect(url: &str) -> AppResult<Self> {
        let client = redis::Client::open(url)?;
        // Verify connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        let _: () = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(Self { client })
    }

    // ============ USERS ============

    pub async fn get_user(&self, username: &str) -> AppResult<Option<User>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let v: Option<String> = c.get(format!("{PREFIX_USER}{username}")).await?;
        match v {
            None => Ok(None),
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        }
    }

    pub async fn put_user(&self, user: &User) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let s = serde_json::to_string(user)?;
        let _: () = c.set(format!("{PREFIX_USER}{}", user.username), s).await?;
        let _: () = c.sadd(SET_USERS, &user.username).await?;
        Ok(())
    }

    pub async fn user_exists(&self, username: &str) -> AppResult<bool> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let n: bool = c.exists(format!("{PREFIX_USER}{username}")).await?;
        Ok(n)
    }

    pub async fn list_users(&self) -> AppResult<Vec<String>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let v: Vec<String> = c.smembers(SET_USERS).await?;
        Ok(v)
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

    // ============ AGENTS ============

    pub async fn put_agent(&self, a: &Agent) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let s = serde_json::to_string(a)?;
        let _: () = c.set(format!("{PREFIX_AGENT}{}", a.id), s).await?;
        let _: () = c.sadd(SET_AGENTS, &a.id).await?;
        Ok(())
    }

    pub async fn get_agent(&self, id: &str) -> AppResult<Option<Agent>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let v: Option<String> = c.get(format!("{PREFIX_AGENT}{id}")).await?;
        match v {
            None => Ok(None),
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        }
    }

    pub async fn list_agents(&self) -> AppResult<Vec<Agent>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let ids: Vec<String> = c.smembers(SET_AGENTS).await?;
        if ids.is_empty() { return Ok(vec![]); }
        let keys: Vec<String> = ids.iter().map(|id| format!("{PREFIX_AGENT}{id}")).collect();
        let values: Vec<Option<String>> = c.mget(&keys).await?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            if let Some(s) = v {
                if let Ok(a) = serde_json::from_str::<Agent>(&s) {
                    out.push(a);
                }
            }
        }
        Ok(out)
    }

    pub async fn delete_agent(&self, id: &str) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let _: () = c.del(format!("{PREFIX_AGENT}{id}")).await?;
        let _: () = c.srem(SET_AGENTS, id).await?;
        Ok(())
    }

    // ============ CUSTOMERS ============

    pub async fn put_customer(&self, x: &Customer) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let s = serde_json::to_string(x)?;
        let _: () = c.set(format!("{PREFIX_CUSTOMER}{}", x.id), s).await?;
        let _: () = c.sadd(SET_CUSTOMERS, &x.id).await?;
        Ok(())
    }

    pub async fn get_customer(&self, id: &str) -> AppResult<Option<Customer>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let v: Option<String> = c.get(format!("{PREFIX_CUSTOMER}{id}")).await?;
        match v {
            None => Ok(None),
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        }
    }

    pub async fn list_customers(&self) -> AppResult<Vec<Customer>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let ids: Vec<String> = c.smembers(SET_CUSTOMERS).await?;
        if ids.is_empty() { return Ok(vec![]); }
        let keys: Vec<String> = ids.iter().map(|id| format!("{PREFIX_CUSTOMER}{id}")).collect();
        let values: Vec<Option<String>> = c.mget(&keys).await?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            if let Some(s) = v {
                if let Ok(x) = serde_json::from_str::<Customer>(&s) {
                    out.push(x);
                }
            }
        }
        Ok(out)
    }

    pub async fn delete_customer(&self, id: &str) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let _: () = c.del(format!("{PREFIX_CUSTOMER}{id}")).await?;
        let _: () = c.srem(SET_CUSTOMERS, id).await?;
        Ok(())
    }

    // ============ INTERACTIONS ============

    pub async fn put_interaction(&self, i: &Interaction) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let s = serde_json::to_string(i)?;
        let _: () = c.set(format!("{PREFIX_INTERACTION}{}", i.id), s).await?;
        let _: () = c.sadd(SET_INTERACTIONS, &i.id).await?;
        Ok(())
    }

    pub async fn get_interaction(&self, id: &str) -> AppResult<Option<Interaction>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let v: Option<String> = c.get(format!("{PREFIX_INTERACTION}{id}")).await?;
        match v {
            None => Ok(None),
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        }
    }

    pub async fn list_interactions(&self) -> AppResult<Vec<Interaction>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let ids: Vec<String> = c.smembers(SET_INTERACTIONS).await?;
        if ids.is_empty() { return Ok(vec![]); }
        let keys: Vec<String> = ids.iter().map(|id| format!("{PREFIX_INTERACTION}{id}")).collect();
        let values: Vec<Option<String>> = c.mget(&keys).await?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            if let Some(s) = v {
                if let Ok(i) = serde_json::from_str::<Interaction>(&s) {
                    out.push(i);
                }
            }
        }
        Ok(out)
    }

    // ============ RUBRICS ============

    pub async fn put_rubric(&self, r: &Rubric) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let s = serde_json::to_string(r)?;
        let _: () = c.set(format!("{PREFIX_RUBRIC}{}", r.id), s).await?;
        let _: () = c.sadd(SET_RUBRICS, &r.id).await?;
        Ok(())
    }

    pub async fn get_rubric(&self, id: &str) -> AppResult<Option<Rubric>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let v: Option<String> = c.get(format!("{PREFIX_RUBRIC}{id}")).await?;
        match v {
            None => Ok(None),
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        }
    }

    pub async fn list_rubrics(&self) -> AppResult<Vec<Rubric>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let ids: Vec<String> = c.smembers(SET_RUBRICS).await?;
        if ids.is_empty() { return Ok(vec![]); }
        let keys: Vec<String> = ids.iter().map(|id| format!("{PREFIX_RUBRIC}{id}")).collect();
        let values: Vec<Option<String>> = c.mget(&keys).await?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            if let Some(s) = v {
                if let Ok(r) = serde_json::from_str::<Rubric>(&s) {
                    out.push(r);
                }
            }
        }
        Ok(out)
    }

    pub async fn ensure_default_rubric(&self) -> AppResult<()> {
        let rubrics = self.list_rubrics().await?;
        if !rubrics.is_empty() {
            return Ok(());
        }
        // Make sure default metrics exist (codes match the seed below)
        self.ensure_default_metrics().await?;
        let metrics = self.list_metrics().await?;
        let by_code = |code: &str| -> String {
            metrics.iter().find(|m| m.code == code)
                .map(|m| m.id.clone())
                .unwrap_or_default()
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

    // ============ SCORES ============

    pub async fn put_score(&self, s: &Score) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(s)?;
        let _: () = c.set(format!("{PREFIX_SCORE}{}", s.interaction_id), json).await?;
        let _: () = c.sadd(SET_SCORES, &s.id).await?;
        Ok(())
    }

    pub async fn get_score_by_interaction(&self, interaction_id: &str) -> AppResult<Option<Score>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let v: Option<String> = c.get(format!("{PREFIX_SCORE}{interaction_id}")).await?;
        match v {
            None => Ok(None),
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        }
    }

    pub async fn list_scores(&self) -> AppResult<Vec<Score>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let ids: Vec<String> = c.smembers(SET_SCORES).await?;
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            // get score by id - we look up by interaction_id indirectly
            if let Ok(Some(s)) = self.get_score_by_id(&id).await {
                out.push(s);
            }
        }
        Ok(out)
    }

    async fn get_score_by_id(&self, _id: &str) -> AppResult<Option<Score>> {
        // We don't index scores by id; iterate via SCAN-like
        let scores = self.scan_scores().await?;
        Ok(scores.into_iter().find(|s| s.id == _id))
    }

    pub async fn scan_scores(&self) -> AppResult<Vec<Score>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let pattern = format!("{PREFIX_SCORE}*");
        let keys: Vec<String> = c.keys(&pattern).await?;
        let mut out = Vec::with_capacity(keys.len());
        for k in keys {
            let v: Option<String> = c.get(&k).await?;
            if let Some(s) = v {
                if let Ok(score) = serde_json::from_str::<Score>(&s) {
                    out.push(score);
                }
            }
        }
        Ok(out)
    }

    // ============ ISSUES ============

    pub async fn put_issue(&self, x: &Issue) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(x)?;
        let _: () = c.set(format!("{PREFIX_ISSUE}{}", x.id), json).await?;
        let _: () = c.sadd(SET_ISSUES, &x.id).await?;
        Ok(())
    }

    pub async fn get_issue(&self, id: &str) -> AppResult<Option<Issue>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let v: Option<String> = c.get(format!("{PREFIX_ISSUE}{id}")).await?;
        match v {
            None => Ok(None),
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        }
    }

    pub async fn list_issues(&self) -> AppResult<Vec<Issue>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let ids: Vec<String> = c.smembers(SET_ISSUES).await?;
        if ids.is_empty() { return Ok(vec![]); }
        let keys: Vec<String> = ids.iter().map(|id| format!("{PREFIX_ISSUE}{id}")).collect();
        let values: Vec<Option<String>> = c.mget(&keys).await?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            if let Some(s) = v {
                if let Ok(i) = serde_json::from_str::<Issue>(&s) {
                    out.push(i);
                }
            }
        }
        Ok(out)
    }

    // ============ DEMO SEED ============

    pub async fn put_metric(&self, m: &MetricDefinition) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let key = format!("{PREFIX_METRIC}{}", m.id);
        let v = serde_json::to_string(m).map_err(|e| AppError::Internal(format!("serialize: {e}")))?;
        let _: () = redis::pipe()
            .atomic()
            .set(&key, v)
            .ignore()
            .sadd(SET_METRICS, &m.id)
            .ignore()
            .hset(SET_METRIC_BY_CODE, &m.code, &m.id)
            .ignore()
            .query_async(&mut c)
            .await?;
        Ok(())
    }

    pub async fn get_metric(&self, id: &str) -> AppResult<Option<MetricDefinition>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let v: Option<String> = c.get(format!("{PREFIX_METRIC}{id}")).await?;
        match v {
            Some(s) => Ok(Some(serde_json::from_str(&s).map_err(|e| AppError::Internal(format!("parse: {e}")))?)),
            None => Ok(None),
        }
    }

    pub async fn get_metric_by_code(&self, code: &str) -> AppResult<Option<MetricDefinition>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let id: Option<String> = c.hget(SET_METRIC_BY_CODE, code).await?;
        match id {
            Some(id) => self.get_metric(&id).await,
            None => Ok(None),
        }
    }

    pub async fn list_metrics(&self) -> AppResult<Vec<MetricDefinition>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let ids: Vec<String> = c.smembers(SET_METRICS).await?;
        if ids.is_empty() { return Ok(vec![]); }
        let keys: Vec<String> = ids.iter().map(|id| format!("{PREFIX_METRIC}{id}")).collect();
        let vals: Vec<Option<String>> = c.mget(keys).await?;
        let mut out = Vec::new();
        for v in vals.into_iter().flatten() {
            if let Ok(m) = serde_json::from_str::<MetricDefinition>(&v) {
                out.push(m);
            }
        }
        Ok(out)
    }

    pub async fn delete_metric(&self, id: &str) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        // fetch first to remove code index
        let m = self.get_metric(id).await?;
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.del(format!("{PREFIX_METRIC}{id}")).ignore();
        pipe.srem(SET_METRICS, id).ignore();
        if let Some(m) = m {
            pipe.hdel(SET_METRIC_BY_CODE, m.code).ignore();
        }
        let _: () = pipe.query_async(&mut c).await?;
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
                critical: true,
                created_at: Utc::now(),
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
                scale_min: None, scale_max: None,
                critical: true,
                created_at: Utc::now(),
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
                required_keywords: vec![],
                scale_min: Some(1.0), scale_max: Some(5.0),
                critical: false,
                created_at: Utc::now(),
            },
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "communication_active_listening".into(),
                title: "گوش دادن فعال".into(),
                description: "توجه کامل به صحبتهای مشتری و پرسشهای تأییدی".into(),
                category: "ارتباط".into(),
                metric_type: MetricType::Scale,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec![], value_scores: Default::default(),
                required_keywords: vec![],
                scale_min: Some(1.0), scale_max: Some(5.0),
                critical: false,
                created_at: Utc::now(),
            },
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "communication_empathy".into(),
                title: "همدلی و احترام".into(),
                description: "ابراز همدلی با مشتری و رعایت احترام".into(),
                category: "ارتباط".into(),
                metric_type: MetricType::Scale,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec![], value_scores: Default::default(),
                required_keywords: vec![],
                scale_min: Some(1.0), scale_max: Some(5.0),
                critical: false,
                created_at: Utc::now(),
            },
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "resolution_first_call".into(),
                title: "حل مشکل در تماس اول".into(),
                description: "آیا مشکل مشتری در همین تماس حل شد؟".into(),
                category: "حل مسئله".into(),
                metric_type: MetricType::Categorical,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec!["حل شد".into(), "پیگیری لازم".into(), "حل نشد".into()],
                value_scores: [("حل شد".to_string(), 100.0), ("پیگیری لازم".to_string(), 60.0), ("حل نشد".to_string(), 0.0)].into_iter().collect(),
                required_keywords: vec![], scale_min: None, scale_max: None,
                critical: false,
                created_at: Utc::now(),
            },
            MetricDefinition {
                id: Uuid::new_v4().to_string(),
                code: "resolution_followup_commitment".into(),
                title: "تعهد پیگیری".into(),
                description: "آیا کارشناس زمان مشخص برای پیگیری به مشتری اعلام کرد؟".into(),
                category: "حل مسئله".into(),
                metric_type: MetricType::Boolean,
                min: None, max: None, higher_is_better: true,
                allowed_values: vec![], value_scores: Default::default(),
                required_keywords: vec![], scale_min: None, scale_max: None,
                critical: false,
                created_at: Utc::now(),
            },
        ];
        for m in &defaults {
            self.put_metric(m).await?;
        }
        Ok(())
    }

    pub async fn seed_demo_data(&self) -> AppResult<()> {
        let existing = self.list_agents().await?;
        if !existing.is_empty() {
            return Ok(());
        }
        let now = Utc::now();
        let agents = vec![
            ("علی رضایی", "بانک", "کارشناس ارشد"),
            ("سارا محمدی", "بیمه", "کارشناس"),
            ("مهدی کریمی", "عمومی", "کارشناس پاسخگویی"),
        ];
        let mut agent_ids = Vec::new();
        for (n, d, p) in agents {
            let a = Agent {
                id: Uuid::new_v4().to_string(),
                name: n.into(), department: d.into(), position: p.into(),
                active: true, created_at: now,
            };
            self.put_agent(&a).await?;
            agent_ids.push(a.id);
        }
        let customers = vec![
            ("رضا احمدی", "09121234567", "تسهیلات", "VIP"),
            ("مریم حسینی", "09129876543", "بیمه عمر", "عادی"),
            ("حسین مرادی", "09125551234", "حساب بانکی", "عادی"),
        ];
        let mut customer_ids = Vec::new();
        for (n, ph, pt, sg) in customers {
            let x = Customer {
                id: Uuid::new_v4().to_string(),
                name: n.into(), phone: ph.into(), product_type: pt.into(), segment: sg.into(),
                notes: "".into(), created_at: now,
            };
            self.put_customer(&x).await?;
            customer_ids.push(x.id);
        }
        let interactions = vec![
            (agent_ids[0].clone(), customer_ids[0].clone(), "تلفن", "درخواست تسهیلات", "سلام، درباره شرایط تسهیلات سوال دارم. کارشناس شرایط را توضیح داد اما زمان پیگیری مشخص نشد."),
            (agent_ids[1].clone(), customer_ids[1].clone(), "چت", "اعتراض به مبلغ بیمه", "مشتری درباره مبلغ بیمه اعتراض داشت. کارشناس با لحن مناسب توضیح داد و درخواست را ثبت کرد."),
            (agent_ids[0].clone(), customer_ids[2].clone(), "حضوری", "اصلاح اطلاعات حساب", "مشتری درخواست اصلاح اطلاعات داشت. احراز هویت ناقص انجام شد و ادامه کار به مراجعه مجدد موکول شد."),
            (agent_ids[2].clone(), customer_ids[0].clone(), "ایمیل", "پیگیری درخواست قبلی", "مشتری پیگیر درخواست قبلی بود. کارشناس سابقه را بررسی و زمان پاسخ بعدی را اعلام کرد."),
            (agent_ids[1].clone(), customer_ids[1].clone(), "تلفن", "سوال درباره خدمات بانکی", "مشتری اطلاعات محصول را خواست و کارشناس پاسخ داد ولی بخشی از اطلاعات نیاز به بررسی مجدد داشت."),
        ];
        for (ag, cu, ch, sb, tr) in interactions {
            let i = Interaction {
                id: Uuid::new_v4().to_string(),
                agent_id: ag, customer_id: cu,
                channel: ch.into(), subject: sb.into(), transcript: tr.into(),
                tags: vec!["نمونه QA".into()],
                created_at: now, updated_at: now,
            };
            self.put_interaction(&i).await?;
        }
        Ok(())
    }

    // =================== KPIs ===================

    pub async fn put_kpi(&self, k: &Kpi) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(k).map_err(|e| AppError::Internal(e.to_string()))?;
        let _: () = c.set(format!("{PREFIX_KPI}{}", k.id), json).await?;
        let _: () = c.sadd(SET_KPIS, &k.id).await?;
        let _: () = c.hset(SET_KPI_BY_CODE, &k.code, &k.id).await?;
        Ok(())
    }

    pub async fn get_kpi(&self, id: &str) -> AppResult<Option<Kpi>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let v: Option<String> = c.get(format!("{PREFIX_KPI}{id}")).await?;
        v.map(|s| serde_json::from_str(&s).map_err(|e| AppError::Internal(e.to_string())))
            .transpose()
    }

    pub async fn get_kpi_by_code(&self, code: &str) -> AppResult<Option<Kpi>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let id: Option<String> = c.hget(SET_KPI_BY_CODE, code).await?;
        match id {
            Some(id) => self.get_kpi(&id).await,
            None => Ok(None),
        }
    }

    pub async fn list_kpis(&self) -> AppResult<Vec<Kpi>> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        let ids: Vec<String> = c.smembers(SET_KPIS).await?;
        if ids.is_empty() { return Ok(vec![]); }
        let keys: Vec<String> = ids.iter().map(|id| format!("{PREFIX_KPI}{id}")).collect();
        let values: Vec<Option<String>> = c.mget(keys).await?;
        let mut kpis = Vec::new();
        for v in values.into_iter().flatten() {
            if let Ok(k) = serde_json::from_str::<Kpi>(&v) {
                kpis.push(k);
            }
        }
        Ok(kpis)
    }

    pub async fn delete_kpi(&self, id: &str) -> AppResult<()> {
        let mut c = self.client.get_multiplexed_async_connection().await?;
        if let Some(k) = self.get_kpi(id).await? {
            let _: () = c.del(format!("{PREFIX_KPI}{id}")).await?;
            let _: () = c.srem(SET_KPIS, id).await?;
            let _: () = c.hdel(SET_KPI_BY_CODE, &k.code).await?;
        }
        Ok(())
    }
}
