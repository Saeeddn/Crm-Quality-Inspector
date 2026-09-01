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
const SET_USERS: &str = "set:users";
const SET_AGENTS: &str = "set:agents";
const SET_CUSTOMERS: &str = "set:customers";
const SET_INTERACTIONS: &str = "set:interactions";
const SET_RUBRICS: &str = "set:rubrics";
const SET_SCORES: &str = "set:scores";
const SET_ISSUES: &str = "set:issues";

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
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
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
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(a) = self.get_agent(&id).await? {
                out.push(a);
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
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(x) = self.get_customer(&id).await? {
                out.push(x);
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
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(i) = self.get_interaction(&id).await? {
                out.push(i);
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
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(r) = self.get_rubric(&id).await? {
                out.push(r);
            }
        }
        Ok(out)
    }

    pub async fn ensure_default_rubric(&self) -> AppResult<()> {
        let rubrics = self.list_rubrics().await?;
        if !rubrics.is_empty() {
            return Ok(());
        }
        let r = Rubric {
            id: Uuid::new_v4().to_string(),
            name: "استاندارد پایه بانک/بیمه".into(),
            department: "عمومی".into(),
            product_type: None,
            channel: None,
            version: 1,
            criteria: vec![
                RubricCriterion { code: "RESP".into(), title: "پاسخگویی".into(), description: "شروع و ادامه تعامل در SLA".into(), weight: 10.0, critical: false },
                RubricCriterion { code: "ACC".into(), title: "دقت اطلاعات".into(), description: "اطلاعات دقیق و منطبق با محصول".into(), weight: 20.0, critical: true },
                RubricCriterion { code: "PRO".into(), title: "حرفه‌ای‌گری".into(), description: "لحن، ادب، همدلی و مالکیت تعامل".into(), weight: 10.0, critical: false },
                RubricCriterion { code: "FCR".into(), title: "حل مسئله".into(), description: "حل در همان تعامل یا ارجاع صحیح".into(), weight: 20.0, critical: false },
                RubricCriterion { code: "FUP".into(), title: "پیگیری".into(), description: "تعهدات و پیگیری وعده‌داده‌شده".into(), weight: 10.0, critical: false },
                RubricCriterion { code: "CMP".into(), title: "انطباق".into(), description: "احراز هویت، محرمانگی، مقررات".into(), weight: 20.0, critical: true },
                RubricCriterion { code: "SAT".into(), title: "تجربه مشتری".into(), description: "شفافیت، سهولت و کاهش تلاش".into(), weight: 10.0, critical: false },
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
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(i) = self.get_issue(&id).await? {
                out.push(i);
            }
        }
        Ok(out)
    }

    // ============ DEMO SEED ============

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
}
