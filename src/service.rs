use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::store::Store;
use chrono::{Duration, Utc};
use uuid::Uuid;

pub struct Service<'a> {
    pub store: &'a Store,
}

fn clamp(v: f64) -> f64 {
    v.max(0.0).min(100.0)
}

fn level_of(overall: f64, critical: bool) -> String {
    if critical {
        "مردود بحرانی".into()
    } else if overall >= 90.0 {
        "عالی".into()
    } else if overall >= 80.0 {
        "خوب".into()
    } else if overall >= 70.0 {
        "قابل قبول".into()
    } else if overall >= 60.0 {
        "نیازمند بهبود".into()
    } else {
        "ضعیف".into()
    }
}

impl<'a> Service<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    // =================== Agents ===================

    pub async fn create_agent(&self, req: CreateAgentRequest) -> AppResult<Agent> {
        if req.name.trim().is_empty() {
            return Err(AppError::Validation("نام کارشناس الزامی است".into()));
        }
        let a = Agent {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            department: req.department,
            position: req.position,
            active: true,
            created_at: Utc::now(),
        };
        self.store.put_agent(&a).await?;
        Ok(a)
    }

    pub async fn update_agent(&self, id: &str, req: UpdateAgentRequest) -> AppResult<Agent> {
        let mut a = self
            .store
            .get_agent(id)
            .await?
            .ok_or_else(|| AppError::NotFound("کارشناس یافت نشد".into()))?;
        if let Some(v) = req.name { a.name = v; }
        if let Some(v) = req.department { a.department = v; }
        if let Some(v) = req.position { a.position = v; }
        if let Some(v) = req.active { a.active = v; }
        self.store.put_agent(&a).await?;
        Ok(a)
    }

    pub async fn delete_agent(&self, id: &str) -> AppResult<()> {
        let interactions = self.store.list_interactions().await?;
        if interactions.iter().any(|i| i.agent_id == id) {
            return Err(AppError::Validation("کارشناس دارای تعامل است؛ ابتدا غیرفعال کنید".into()));
        }
        self.store.delete_agent(id).await?;
        Ok(())
    }

    // =================== Customers ===================

    pub async fn create_customer(&self, req: CreateCustomerRequest) -> AppResult<Customer> {
        if req.name.trim().is_empty() {
            return Err(AppError::Validation("نام مشتری الزامی است".into()));
        }
        let c = Customer {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            phone: req.phone,
            product_type: req.product_type,
            segment: req.segment,
            notes: req.notes,
            created_at: Utc::now(),
        };
        self.store.put_customer(&c).await?;
        Ok(c)
    }

    pub async fn update_customer(&self, id: &str, req: UpdateCustomerRequest) -> AppResult<Customer> {
        let mut c = self
            .store
            .get_customer(id)
            .await?
            .ok_or_else(|| AppError::NotFound("مشتری یافت نشد".into()))?;
        if let Some(v) = req.name { c.name = v; }
        if let Some(v) = req.phone { c.phone = v; }
        if let Some(v) = req.product_type { c.product_type = v; }
        if let Some(v) = req.segment { c.segment = v; }
        if let Some(v) = req.notes { c.notes = v; }
        self.store.put_customer(&c).await?;
        Ok(c)
    }

    pub async fn delete_customer(&self, id: &str) -> AppResult<()> {
        let interactions = self.store.list_interactions().await?;
        if interactions.iter().any(|i| i.customer_id == id) {
            return Err(AppError::Validation("مشتری دارای سابقه تعامل است".into()));
        }
        self.store.delete_customer(id).await?;
        Ok(())
    }

    // =================== Interactions ===================

    pub async fn create_interaction(&self, req: CreateInteractionRequest) -> AppResult<Interaction> {
        if self.store.get_agent(&req.agent_id).await?.is_none() {
            return Err(AppError::Validation("کارشناس یافت نشد".into()));
        }
        if self.store.get_customer(&req.customer_id).await?.is_none() {
            return Err(AppError::Validation("مشتری یافت نشد".into()));
        }
        if req.transcript.trim().is_empty() {
            return Err(AppError::Validation("متن تعامل برای ارزیابی لازم است".into()));
        }
        let now = Utc::now();
        let i = Interaction {
            id: Uuid::new_v4().to_string(),
            agent_id: req.agent_id,
            customer_id: req.customer_id,
            channel: req.channel,
            subject: req.subject,
            transcript: req.transcript,
            tags: req.tags,
            created_at: now,
            updated_at: now,
        };
        self.store.put_interaction(&i).await?;
        Ok(i)
    }

    // =================== Rubrics ===================

    pub async fn create_rubric(&self, req: CreateRubricRequest) -> AppResult<Rubric> {
        if req.criteria.is_empty() {
            return Err(AppError::Validation("حداقل یک معیار لازم است".into()));
        }
        let sum: f64 = req.criteria.iter().map(|c| c.weight).sum();
        if (sum - 100.0).abs() > 0.01 {
            return Err(AppError::Validation(format!(
                "مجموع وزن معیارها باید ۱۰۰ باشد؛ اکنون {}",
                sum
            )));
        }
        let r = Rubric {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            department: req.department,
            product_type: req.product_type,
            channel: req.channel,
            version: 1,
            criteria: req.criteria,
            active: true,
            created_at: Utc::now(),
        };
        self.store.put_rubric(&r).await?;
        Ok(r)
    }

    // =================== Scoring ===================

    pub async fn select_rubric(&self, interaction: &Interaction, override_id: Option<&str>) -> AppResult<Rubric> {
        if let Some(id) = override_id {
            return self
                .store
                .get_rubric(id)
                .await?
                .ok_or_else(|| AppError::NotFound("استاندارد یافت نشد".into()));
        }
        let customer = self.store.get_customer(&interaction.customer_id).await?;
        let agent = self.store.get_agent(&interaction.agent_id).await?;
        let rubrics = self.store.list_rubrics().await?;
        for r in rubrics {
            if !r.active { continue; }
            let dept_ok = r.department == "عمومی"
                || agent.as_ref().map(|a| a.department == r.department).unwrap_or(false);
            let prod_ok = r.product_type.as_ref().map_or(true, |p| {
                customer.as_ref().map(|c| &c.product_type == p).unwrap_or(false)
            });
            let chan_ok = r.channel.as_ref().map_or(true, |c| &interaction.channel == c);
            if dept_ok && prod_ok && chan_ok {
                return Ok(r);
            }
        }
        Err(AppError::NotFound("استاندارد مناسب پیدا نشد".into()))
    }

    pub async fn score_interaction(&self, req: ScoreRequest) -> AppResult<Score> {
        let interaction = self
            .store
            .get_interaction(&req.interaction_id)
            .await?
            .ok_or_else(|| AppError::NotFound("تعامل یافت نشد".into()))?;
        let rubric = self.select_rubric(&interaction, req.rubric_id.as_deref()).await?;
        if req.scores.len() != rubric.criteria.len() {
            return Err(AppError::Validation(format!(
                "تعداد نمرات باید {} باشد",
                rubric.criteria.len()
            )));
        }
        if req.scores.iter().any(|v| !v.is_finite() || *v < 0.0 || *v > 100.0) {
            return Err(AppError::Validation("هر نمره باید بین ۰ و ۱۰۰ باشد".into()));
        }
        let mut total = 0.0;
        let mut critical = false;
        let mut reasons = Vec::new();
        for (i, c) in rubric.criteria.iter().enumerate() {
            let s = clamp(req.scores[i]);
            total += s * c.weight / 100.0;
            if c.critical && s < 60.0 {
                critical = true;
                reasons.push(format!("{}: نمره {} کمتر از حد بحرانی ۶۰", c.title, s));
            }
        }
        let overall = (total * 100.0).round() / 100.0;
        let level = level_of(overall, critical);
        let score = Score {
            id: Uuid::new_v4().to_string(),
            interaction_id: req.interaction_id.clone(),
            rubric_id: rubric.id.clone(),
            overall_score: overall,
            level,
            dimension_scores: req.scores,
            critical_fail: critical,
            critical_fail_reasons: reasons,
            evaluator: req.evaluator.unwrap_or_else(|| "system".into()),
            notes: req.notes,
            created_at: Utc::now(),
        };
        self.store.put_score(&score).await?;
        self.auto_create_issues(&interaction, &score, &rubric).await?;
        Ok(score)
    }

    async fn auto_create_issues(&self, interaction: &Interaction, score: &Score, _rubric: &Rubric) -> AppResult<()> {
        let existing = self.store.list_issues().await?;
        let mut to_create: Vec<(String, String, String)> = Vec::new();
        if score.critical_fail {
            to_create.push((
                "بحرانی".into(),
                "انطباق/دقت".into(),
                format!("شکست معیار بحرانی: {}", score.critical_fail_reasons.join("؛ ")),
            ));
        } else if score.overall_score < 60.0 {
            to_create.push((
                "بالا".into(),
                "کیفیت کلی".into(),
                format!("امتیاز کیفیت {} کمتر از حد هشدار ۶۰", score.overall_score),
            ));
        } else if score.overall_score < 70.0 {
            to_create.push((
                "متوسط".into(),
                "بهبود".into(),
                format!("تعامل نیازمند برنامه بهبود است؛ امتیاز {}", score.overall_score),
            ));
        }
        for (sev, cat, desc) in to_create {
            let already = existing
                .iter()
                .any(|x| x.interaction_id == interaction.id && x.status != "بسته" && x.category == cat);
            if already {
                continue;
            }
            let due_days = if sev == "بحرانی" { 1 } else { 3 };
            let issue = Issue {
                id: Uuid::new_v4().to_string(),
                interaction_id: interaction.id.clone(),
                agent_id: interaction.agent_id.clone(),
                severity: sev,
                category: cat,
                description: desc,
                status: "باز".into(),
                root_cause: None,
                corrective_action: None,
                due_at: Some(Utc::now() + Duration::days(due_days)),
                created_at: Utc::now(),
                resolved_at: None,
            };
            self.store.put_issue(&issue).await?;
        }
        Ok(())
    }

    // =================== Issues ===================

    pub async fn resolve_issue(&self, id: &str, req: ResolveIssueRequest) -> AppResult<Issue> {
        let mut issue = self
            .store
            .get_issue(id)
            .await?
            .ok_or_else(|| AppError::NotFound("ایراد یافت نشد".into()))?;
        if req.root_cause.trim().is_empty() {
            return Err(AppError::Validation("علت ریشه‌ای الزامی است".into()));
        }
        issue.status = "بسته".into();
        issue.root_cause = Some(req.root_cause);
        issue.corrective_action = Some(req.corrective_action);
        issue.resolved_at = Some(Utc::now());
        self.store.put_issue(&issue).await?;
        Ok(issue)
    }

    // =================== Recommendations ===================

    pub async fn compute_recommendations(&self) -> AppResult<Vec<serde_json::Value>> {
        let interactions = self.store.list_interactions().await?;
        let scores = self.store.scan_scores().await?;
        let customers = self.store.list_customers().await?;
        let mut out = Vec::new();
        for i in interactions {
            if scores.iter().any(|s| s.interaction_id == i.id) {
                continue;
            }
            let age = (Utc::now() - i.created_at).num_hours();
            let priority = if age >= 24 {
                "بالا"
            } else if age >= 8 {
                "متوسط"
            } else {
                "کم"
            };
            let customer_name = customers
                .iter()
                .find(|c| c.id == i.customer_id)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            out.push(serde_json::json!({
                "interaction_id": i.id,
                "customer_name": customer_name,
                "agent_id": i.agent_id,
                "reason": format!("تعامل هنوز ارزیابی نشده و {} ساعت از ثبت آن گذشته", age),
                "suggested_action": "ارزیابی تعامل طبق روبریک مربوطه",
                "priority": priority,
            }));
        }
        Ok(out)
    }

    // =================== Dashboard ===================

    pub async fn dashboard(&self) -> AppResult<serde_json::Value> {
        let agents = self.store.list_agents().await?;
        let customers = self.store.list_customers().await?;
        let interactions = self.store.list_interactions().await?;
        let scores = self.store.scan_scores().await?;
        let issues = self.store.list_issues().await?;

        let n = scores.len();
        let avg = if n > 0 {
            scores.iter().map(|s| s.overall_score).sum::<f64>() / n as f64
        } else {
            0.0
        };
        let critical = scores.iter().filter(|s| s.critical_fail).count();
        let open = issues.iter().filter(|x| x.status == "باز").count();
        let coverage = if interactions.is_empty() {
            0.0
        } else {
            n as f64 * 100.0 / interactions.len() as f64
        };
        let grade = if critical > 0 {
            "نیازمند اقدام"
        } else if avg >= 90.0 {
            "A+"
        } else if avg >= 80.0 {
            "A"
        } else if avg >= 70.0 {
            "B"
        } else if avg >= 60.0 {
            "C"
        } else {
            "D"
        };
        Ok(serde_json::json!({
            "agent_count": agents.len(),
            "customer_count": customers.len(),
            "interaction_count": interactions.len(),
            "scored_count": n,
            "average_score": (avg * 100.0).round() / 100.0,
            "open_issues": open,
            "critical_failures": critical,
            "coverage": (coverage * 100.0).round() / 100.0,
            "quality_grade": grade,
        }))
    }

    pub async fn agent_report(&self, id: &str) -> AppResult<serde_json::Value> {
        let agent = self.store.get_agent(id).await?;
        let interactions = self.store.list_interactions().await?;
        let scores = self.store.scan_scores().await?;
        let agent_scores: Vec<&Score> = scores
            .iter()
            .filter(|s| {
                interactions
                    .iter()
                    .find(|i| i.id == s.interaction_id)
                    .map(|i| i.agent_id == id)
                    .unwrap_or(false)
            })
            .collect();
        let n = agent_scores.len();
        let avg = if n > 0 {
            agent_scores.iter().map(|s| s.overall_score).sum::<f64>() / n as f64
        } else {
            0.0
        };
        let critical = agent_scores.iter().filter(|s| s.critical_fail).count();
        Ok(serde_json::json!({
            "agent": agent,
            "scored_interactions": n,
            "average_score": avg,
            "critical_failures": critical,
            "scores": agent_scores,
        }))
    }
}
