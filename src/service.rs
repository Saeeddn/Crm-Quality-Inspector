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

// =====================
// KPI Measurement Engine
// =====================

/// کلمات کلیدی منفی پایه (برای تشخیص ریسک)
const NEGATIVE_KEYWORDS: &[&str] = &[
    "شکایت", "ناراضی", "عصبانی", "ناراحت", "افتضاح", "بد", "ضعیف",
    "مسئله", "مشکل", "خطا", "اشتباه", "تاخیر", "طولانی", "نمیتونم",
    "نمی‌توانم", "نمیشه", "نمی‌شود", "پیگیری", "مرجوع",
];

/// کلمات کلیدی احوالپرسی
const GREETING_KEYWORDS: &[&str] = &[
    "سلام", "درود", "صبح بخیر", "عصر بخیر", "وقت بخیر", "احتراما",
    "متشکرم", "ممنون", "تشکر", "خوشحال", "خوشوقت",
];

/// کلمات کلیدی پایان مناسب
const CLOSING_KEYWORDS: &[&str] = &[
    "موفق باشید", "روز خوب", "موفق", "خداحافظ", "بدرود",
    "پیگیری میکنم", "پیگیری می‌کنم", "در اسرع وقت", "سپاسگزارم",
];

/// کلمات کلیدی عذرخواهی/همدلی
const EMPATHY_KEYWORDS: &[&str] = &[
    "ببخشید", "متاسفم", "متأسفم", "معذرت", "درک میکنم", "درک می‌کنم",
    "حق با شماست", "حق دارید", "نگرانی شما",
];

/// شمارش تعداد وقوع یک یا چند کلمه در متن
fn count_keyword_occurrences(text: &str, pattern: &str) -> usize {
    // اگر pattern شامل کاما باشد، چند کلمه
    if pattern.contains(',') {
        let words: Vec<&str> = pattern.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        words.iter().map(|w| count_keyword_occurrences(text, w)).sum()
    } else {
        text.matches(pattern).count()
    }
}

/// اندازه‌گیری یک KPI روی transcript
fn measure_kpi(kpi: &Kpi, transcript: &str) -> KpiMeasurement {
    let t = transcript;
    let evidence_fa = |count: usize, pattern: &str| -> String {
        if count == 0 { format!("یافت نشد: «{}»", pattern) }
        else { format!("یافت شد {} بار: «{}»", to_fa_num(count), pattern) }
    };
    let mut m = KpiMeasurement {
        kpi_id: kpi.id.clone(),
        kpi_code: kpi.code.clone(),
        kpi_name: kpi.name.clone(),
        kind: kpi.kind.clone(),
        measured: 0.0,
        expected: kpi.threshold,
        score: 0.0,
        weight: kpi.weight,
        weighted: 0.0,
        critical: kpi.critical,
        critical_fail: false,
        evidence: String::new(),
    };
    match kpi.kind {
        KpiKind::KeywordCount => {
            let pat = kpi.pattern.as_deref().unwrap_or("");
            let cnt = count_keyword_occurrences(t, pat) as f64;
            m.measured = cnt;
            let thr = kpi.threshold.unwrap_or(1.0);
            // اگر cnt >= thr: 100. اگر cnt < thr: linear کاهش
            m.score = clamp((cnt / thr.max(1.0)) * 100.0);
            m.evidence = evidence_fa(cnt as usize, pat);
        }
        KpiKind::KeywordPresence => {
            let pat = kpi.pattern.as_deref().unwrap_or("");
            let found = t.contains(pat);
            m.measured = if found { 1.0 } else { 0.0 };
            m.score = if found { 100.0 } else { 0.0 };
            m.evidence = if found { format!("یافت شد: «{}»", pat) } else { format!("یافت نشد: «{}»", pat) };
        }
        KpiKind::TextLength => {
            // طول بر اساس تعداد کلمات فارسی
            let words: Vec<&str> = t.split_whitespace().filter(|w| !w.is_empty()).collect();
            let cnt = words.len() as f64;
            m.measured = cnt;
            let thr = kpi.threshold.unwrap_or(50.0);
            m.score = clamp((cnt / thr.max(1.0)) * 100.0);
            m.evidence = format!("{} کلمه", to_fa_num(cnt as usize));
        }
        KpiKind::KeywordRatio => {
            let pat = kpi.pattern.as_deref().unwrap_or("");
            let total_pat = kpi.ratio_total_pattern.as_deref().unwrap_or("");
            let target = count_keyword_occurrences(t, pat) as f64;
            let total = if total_pat.is_empty() {
                t.split_whitespace().count() as f64
            } else {
                count_keyword_occurrences(t, total_pat) as f64
            };
            let ratio = if total > 0.0 { target / total } else { 0.0 };
            m.measured = ratio * 100.0; // درصد
            let thr = kpi.threshold.unwrap_or(10.0);
            // برای کلمات منفی: کمتر بهتر
            // برای کلمات مثبت: بیشتر بهتر
            let is_negative = pat.contains("شکایت") || pat.contains("ناراضی")
                || pat.contains("عصبانی") || pat.contains("ناراحت")
                || pat.contains("مشکل") || pat.contains("خطا");
            m.score = if is_negative {
                // هرچه ratio کمتر، نمره بیشتر
                if ratio == 0.0 { 100.0 }
                else { clamp(100.0 - (ratio * 100.0) * (100.0 / thr.max(0.1))) }
            } else {
                clamp((ratio * 100.0) / thr.max(0.1))
            };
            m.evidence = format!("{} از {} ({}٪)", to_fa_num(target as usize), to_fa_num(total as usize),
                to_fa_num_f(ratio * 100.0, 1));
        }
        KpiKind::ResponseTime => {
            // برای حالت بدون داده واقعی، از طول متن تخمین می‌زنیم
            let est = kpi.threshold.unwrap_or(30000.0); // ms
            m.measured = est;
            m.score = 100.0;
            m.evidence = "زمان پاسخ ثبت نشده (نیاز به سیستم تلفنی)".into();
        }
        KpiKind::ManualRange => {
            // ManualRange توسط ارزیاب تنظیم می‌شود - در حالت خودکار 50 پیش‌فرض
            m.measured = 50.0;
            m.score = 50.0;
            m.evidence = "نیاز به ارزیابی دستی".into();
        }
    }
    if m.score < 60.0 && m.critical { m.critical_fail = true; }
    m.weighted = m.score * m.weight / 100.0;
    m
}

/// محاسبه نمره کلی از همه KPIs
fn aggregate_kpi_score(measurements: &[KpiMeasurement]) -> (f64, bool) {
    if measurements.is_empty() { return (0.0, false); }
    let total_weight: f64 = measurements.iter().map(|m| m.weight).sum();
    if total_weight <= 0.0 { return (0.0, false); }
    let sum: f64 = measurements.iter().map(|m| m.weighted).sum();
    let overall = sum / total_weight * 100.0;
    let critical_fail = measurements.iter().any(|m| m.critical_fail);
    (clamp(overall), critical_fail)
}

fn to_fa_num(n: usize) -> String {
    let mut s = String::new();
    for c in n.to_string().chars() {
        s.push(match c {
            '0' => '۰', '1' => '۱', '2' => '۲', '3' => '۳', '4' => '۴',
            '5' => '۵', '6' => '۶', '7' => '۷', '8' => '۸', '9' => '۹',
            _ => c,
        });
    }
    s
}

fn to_fa_num_f(n: f64, decimals: usize) -> String {
    let s = format!("{:.*}", decimals, n);
    to_fa_num_str(&s)
}

fn to_fa_num_str(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        out.push(match c {
            '0' => '۰', '1' => '۱', '2' => '۲', '3' => '۳', '4' => '۴',
            '5' => '۵', '6' => '۶', '7' => '۷', '8' => '۸', '9' => '۹',
            '.' => '٫', _ => c,
        });
    }
    out
}

fn level_for(score: f64) -> String {
    if score >= 90.0 { "عالی".into() }
    else if score >= 75.0 { "خوب".into() }
    else if score >= 60.0 { "نیازمند بهبود".into() }
    else { "ضعیف".into() }
}

// =====================
// Risk Score Engine (Predictive QA Queue)
// =====================

/// کلمات منفی برای تحلیل ریسک (ساده)
fn count_negative_words(text: &str) -> usize {
    NEGATIVE_KEYWORDS.iter().map(|k| text.matches(k).count()).sum()
}

fn count_greeting(text: &str) -> usize {
    GREETING_KEYWORDS.iter().map(|k| text.matches(k).count()).sum()
}

fn count_closing(text: &str) -> usize {
    CLOSING_KEYWORDS.iter().map(|k| text.matches(k).count()).sum()
}

fn count_empathy(text: &str) -> usize {
    EMPATHY_KEYWORDS.iter().map(|k| text.matches(k).count()).sum()
}

impl<'a> Service<'a> {
    // =================== KPI ===================

    pub async fn list_kpis(&self) -> AppResult<Vec<Kpi>> {
        self.store.list_kpis().await
    }

    pub async fn get_kpi(&self, id: &str) -> AppResult<Kpi> {
        self.store.get_kpi(id).await?
            .ok_or_else(|| AppError::NotFound("kpi".into()))
    }

    pub async fn create_kpi(&self, req: KpiCreateReq) -> AppResult<Kpi> {
        if self.store.get_kpi_by_code(&req.code).await?.is_some() {
            return Err(AppError::Conflict(format!("کد «{}» تکراری است", req.code)));
        }
        if req.weight < 0.0 || req.weight > 100.0 {
            return Err(AppError::BadRequest("وزن باید بین 0 تا 100 باشد".into()));
        }
        let kpi = Kpi {
            id: Uuid::new_v4().to_string(),
            code: req.code,
            name: req.name,
            kind: req.kind,
            description: req.description,
            pattern: req.pattern,
            threshold: req.threshold,
            ratio_total_pattern: req.ratio_total_pattern,
            weight: req.weight,
            critical: req.critical,
            active: true,
            created_at: Utc::now(),
        };
        self.store.put_kpi(&kpi).await?;
        Ok(kpi)
    }

    pub async fn delete_kpi(&self, id: &str) -> AppResult<()> {
        self.store.delete_kpi(id).await
    }

    pub async fn toggle_kpi(&self, id: &str, active: bool) -> AppResult<Kpi> {
        let mut k = self.get_kpi(id).await?;
        k.active = active;
        self.store.put_kpi(&k).await?;
        Ok(k)
    }

    /// محاسبه امتیاز خودکار یک تعامل بر اساس همه KPIs
    pub async fn auto_score(&self, interaction_id: &str) -> AppResult<serde_json::Value> {
        let interaction = self.store.get_interaction(interaction_id).await?
            .ok_or_else(|| AppError::NotFound("interaction".into()))?;
        let kpis = self.store.list_kpis().await?;
        let active_kpis: Vec<&Kpi> = kpis.iter().filter(|k| k.active).collect();
        if active_kpis.is_empty() {
            return Err(AppError::BadRequest("هیچ KPI فعالی تعریف نشده".into()));
        }
        let measurements: Vec<KpiMeasurement> = active_kpis.iter()
            .map(|k| measure_kpi(k, &interaction.transcript))
            .collect();
        let (overall, critical_fail) = aggregate_kpi_score(&measurements);
        let level = level_for(overall);
        let result = serde_json::json!({
            "measurements": measurements,
            "overall_score": overall,
            "level": level,
            "critical_fail": critical_fail,
        });
        Ok(result)
    }

    pub async fn auto_score_and_save(&self, interaction_id: &str) -> AppResult<Score> {
        let interaction = self.store.get_interaction(interaction_id).await?
            .ok_or_else(|| AppError::NotFound("interaction".into()))?;
        let kpis = self.store.list_kpis().await?;
        let active_kpis: Vec<&Kpi> = kpis.iter().filter(|k| k.active).collect();
        if active_kpis.is_empty() {
            return Err(AppError::BadRequest("هیچ KPI فعالی تعریف نشده".into()));
        }
        let measurements: Vec<KpiMeasurement> = active_kpis.iter()
            .map(|k| measure_kpi(k, &interaction.transcript))
            .collect();
        let (overall, critical_fail) = aggregate_kpi_score(&measurements);
        let level = level_for(overall);
        let dim_scores: Vec<f64> = active_kpis.iter().enumerate()
            .map(|(i, _)| measurements[i].score)
            .collect();
        let now = Utc::now();
        let score = Score {
            id: Uuid::new_v4().to_string(),
            interaction_id: interaction_id.to_string(),
            rubric_id: String::new(),
            overall_score: overall,
            level,
            dimension_scores: dim_scores,
            critical_fail,
            critical_fail_reasons: vec![],
            evaluator: "auto_qa".into(),
            notes: format!("امتیازدهی خودکار توسط {} KPI", active_kpis.len()),
            created_at: now,
        };
        self.store.put_score(&score).await?;
        Ok(score)
    }

    /// تولید KPIs پیشفرض فارسی برای شروع سریع
    pub async fn seed_default_kpis(&self) -> AppResult<Vec<Kpi>> {
        let defaults: Vec<KpiCreateReq> = vec![
            KpiCreateReq {
                code: "greeting".into(), name: "احوالپرسی".into(),
                kind: KpiKind::KeywordPresence,
                description: "شروع مکالمه با سلام یا احوالپرسی مناسب".into(),
                pattern: Some("سلام".into()),
                threshold: None, ratio_total_pattern: None,
                weight: 10.0, critical: false,
            },
            KpiCreateReq {
                code: "greeting_count".into(), name: "تعداد احوالپرسی".into(),
                kind: KpiKind::KeywordCount,
                description: "حداقل ۲ مورد احوالپرسی در مکالمه".into(),
                pattern: Some("سلام,متشکرم,ممنون,صبح بخیر,عصر بخیر".into()),
                threshold: Some(2.0), ratio_total_pattern: None,
                weight: 10.0, critical: false,
            },
            KpiCreateReq {
                code: "empathy".into(), name: "همدلی و عذرخواهی".into(),
                kind: KpiKind::KeywordCount,
                description: "نشان دادن درک و عذرخواهی در صورت نیاز".into(),
                pattern: Some("ببخشید,متاسفم,درک میکنم,حق دارید,حق با شماست".into()),
                threshold: Some(1.0), ratio_total_pattern: None,
                weight: 15.0, critical: false,
            },
            KpiCreateReq {
                code: "closing".into(), name: "پایان مناسب".into(),
                kind: KpiKind::KeywordCount,
                description: "بستن مکالمه با تشکر و آرزوی موفقیت".into(),
                pattern: Some("موفق باشید,روز خوب,خداحافظ,سپاسگزارم,پیگیری میکنم".into()),
                threshold: Some(1.0), ratio_total_pattern: None,
                weight: 10.0, critical: false,
            },
            KpiCreateReq {
                code: "min_length".into(), name: "طول مکالمه".into(),
                kind: KpiKind::TextLength,
                description: "حداقل ۳۰ کلمه در مکالمه".into(),
                pattern: None,
                threshold: Some(30.0), ratio_total_pattern: None,
                weight: 5.0, critical: false,
            },
            KpiCreateReq {
                code: "negative_ratio".into(), name: "نسبت کلمات منفی".into(),
                kind: KpiKind::KeywordRatio,
                description: "نسبت کلمات منفی به کل - هرچه کمتر بهتر".into(),
                pattern: Some("شکایت,ناراضی,عصبانی,ناراحت,مشکل,خطا,نمیتونم".into()),
                threshold: Some(5.0), ratio_total_pattern: None,
                weight: 25.0, critical: true,
            },
            KpiCreateReq {
                code: "agent_self_intro".into(), name: "معرفی کارشناس".into(),
                kind: KpiKind::KeywordPresence,
                description: "معرفی نام یا واحد در شروع مکالمه".into(),
                pattern: Some("کارشناس".into()),
                threshold: None, ratio_total_pattern: None,
                weight: 10.0, critical: false,
            },
        ];
        let mut created = vec![];
        for d in defaults {
            if self.store.get_kpi_by_code(&d.code).await?.is_none() {
                let k = self.create_kpi(d).await?;
                created.push(k);
            }
        }
        Ok(created)
    }

    // =================== Predictive QA Queue ===================

    pub async fn recommendations(&self) -> AppResult<Vec<QaRecommendation>> {
        let (interactions, scores, agents, customers) = tokio::try_join!(
            self.store.list_interactions(),
            self.store.list_scores(),
            self.store.list_agents(),
            self.store.list_customers(),
        )?;
        let mut recs = Vec::new();
        for i in interactions {
            if scores.iter().any(|s| s.interaction_id == i.id) { continue; }
            let mut factors = Vec::new();
            let mut risk = 0.0;
            // 1. هنوز ارزیابی نشده
            factors.push(RiskFactor {
                code: "missing_score".into(), label: "ارزیابی نشده".into(),
                points: 25.0,
                reason: "این تعامل هنوز توسط QA بررسی نشده".into(),
            });
            risk += 25.0;
            // 2. مشتری VIP
            if let Some(c) = customers.iter().find(|c| c.id == i.customer_id) {
                if c.segment.contains("VIP") || c.segment.contains("شرکتی") {
                    factors.push(RiskFactor {
                        code: "vip_customer".into(), label: "مشتری ویژه".into(),
                        points: 25.0,
                        reason: format!("مشتری در بخش {}", c.segment),
                    });
                    risk += 25.0;
                }
            }
            // 3. میانگین امتیاز پایین کارشناس
            let agent_scores: Vec<&Score> = scores.iter()
                .filter(|s| s.interaction_id.starts_with(&i.agent_id) || s.interaction_id == i.id)
                .collect();
            // Use overall_score from all scores of this agent (by looking up interaction.agent_id in interactions)
            let agent_avg = {
                let mut matching: Vec<f64> = vec![];
                for s in &scores {
                    // find interaction for this score
                    if let Ok(Some(intr)) = self.store.get_interaction(&s.interaction_id).await {
                        if intr.agent_id == i.agent_id {
                            matching.push(s.overall_score);
                        }
                    }
                }
                if matching.is_empty() { None } else {
                    Some(matching.iter().sum::<f64>() / matching.len() as f64)
                }
            };
            if let Some(avg) = agent_avg {
                let avg: f64 = agent_scores.iter().map(|s| s.overall_score).sum::<f64>()
                    / agent_scores.len() as f64;
                if avg < 70.0 {
                    factors.push(RiskFactor {
                        code: "low_agent_avg".into(), label: "عملکرد ضعیف کارشناس".into(),
                        points: 20.0,
                        reason: format!("میانگین امتیاز کارشناس: {}", to_fa_num_f(avg, 1)),
                    });
                    risk += 20.0;
                } else if avg < 80.0 {
                    factors.push(RiskFactor {
                        code: "agent_below_avg".into(), label: "عملکرد متوسط".into(),
                        points: 10.0,
                        reason: format!("میانگین امتیاز: {}", to_fa_num_f(avg, 1)),
                    });
                    risk += 10.0;
                }
            } else {
                // کارشناس بدون هیچ ارزیابی
                factors.push(RiskFactor {
                    code: "new_agent".into(), label: "کارشناس جدید".into(),
                    points: 15.0,
                    reason: "این کارشناس هنوز ارزیابی نشده".into(),
                });
                risk += 15.0;
            }
            // 4. کانال پیچیده
            if i.channel == "شکایت" || i.channel == "ایمیل" {
                factors.push(RiskFactor {
                    code: "complex_channel".into(), label: "کانال حساس".into(),
                    points: 10.0,
                    reason: format!("کانال {} نیاز به دقت بیشتر دارد", i.channel),
                });
                risk += 10.0;
            }
            // 5. تحلیل ریسک متن
            let neg_count = count_negative_words(&i.transcript);
            if neg_count >= 3 {
                factors.push(RiskFactor {
                    code: "sentiment_negative".into(), label: "لحن منفی".into(),
                    points: 20.0,
                    reason: format!("{} کلمه منفی در متن یافت شد", to_fa_num(neg_count)),
                });
                risk += 20.0;
            } else if neg_count >= 1 {
                factors.push(RiskFactor {
                    code: "sentiment_warning".into(), label: "نشانه‌های نارضایتی".into(),
                    points: 10.0,
                    reason: format!("{} کلمه منفی", to_fa_num(neg_count)),
                });
                risk += 10.0;
            }
            // 6. بدون احوالپرسی
            if count_greeting(&i.transcript) == 0 {
                factors.push(RiskFactor {
                    code: "no_greeting".into(), label: "بدون احوالپرسی".into(),
                    points: 10.0,
                    reason: "مکالمه با سلام شروع نشده".into(),
                });
                risk += 10.0;
            }
            // 7. بدون پایان مناسب
            if count_closing(&i.transcript) == 0 {
                factors.push(RiskFactor {
                    code: "no_closing".into(), label: "بدون پایان مناسب".into(),
                    points: 5.0,
                    reason: "مکالمه با تشکر پایان نیافته".into(),
                });
                risk += 5.0;
            }
            let risk = clamp(risk);
            let priority = if risk >= 70.0 { "بالا" }
                else if risk >= 40.0 { "متوسط" }
                else { "پایین" }.to_string();
            let suggested = if neg_count > 0 { "بررسی فوری + تماس پیگیری" }
                else if priority == "بالا" { "اولویت ارزیابی در روز جاری" }
                else { "ارزیابی در نوبت بعدی" }.to_string();
            let reason = factors.iter()
                .map(|f| f.label.clone())
                .collect::<Vec<_>>()
                .join(" + ");
            let agent_name = agents.iter().find(|a| a.id == i.agent_id)
                .map(|a| a.name.clone()).unwrap_or_default();
            let customer_name = customers.iter().find(|c| c.id == i.customer_id)
                .map(|c| c.name.clone()).unwrap_or_default();
            recs.push(QaRecommendation {
                interaction_id: i.id,
                subject: i.subject,
                agent_id: i.agent_id,
                agent_name,
                customer_id: i.customer_id,
                customer_name,
                channel: i.channel,
                created_at: i.created_at,
                risk_score: risk,
                priority,
                reason,
                factors,
                suggested_action: suggested,
            });
        }
        recs.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(recs)
    }
}

// =====================
// Original Service
// =====================

// Metric Engine
// =====================

/// Convert a raw measurement into a 0-100 score using the metric's
/// measurement type and configuration. Returns (score, failed_critical).
pub fn evaluate_metric(
    metric: &MetricDefinition,
    input: &MeasurementInput,
) -> AppResult<(f64, bool)> {
    let s = match (&metric.metric_type, input) {
        (MetricType::Boolean, MeasurementInput::Bool(b)) => if *b { 100.0 } else { 0.0 },
        (MetricType::Boolean, _) => {
            return Err(AppError::BadRequest(format!(
                "متریک «{}» فقط مقدار true/false می‌پذیرد", metric.title
            )))
        }

        (MetricType::Numeric, MeasurementInput::Number(n)) => {
            let lo = metric.min.unwrap_or(0.0);
            let hi = metric.max.unwrap_or(100.0);
            if hi == lo { return Ok((50.0, false)); }
            let raw = if metric.higher_is_better {
                (n - lo) / (hi - lo)
            } else {
                (hi - n) / (hi - lo)
            };
            clamp(raw * 100.0)
        }
        (MetricType::Numeric, _) => {
            return Err(AppError::BadRequest(format!(
                "متریک «{}» فقط عدد می‌پذیرد", metric.title
            )))
        }

        (MetricType::Categorical, MeasurementInput::Text(t)) => {
            if !metric.allowed_values.contains(t) {
                return Err(AppError::BadRequest(format!(
                    "مقدار «{}» برای متریک «{}» مجاز نیست. مقادیر مجاز: {}",
                    t, metric.title, metric.allowed_values.join("، ")
                )));
            }
            metric.value_scores.get(t).copied().unwrap_or(50.0)
        }
        (MetricType::Categorical, _) => {
            return Err(AppError::BadRequest(format!(
                "متریک «{}» فقط متن می‌پذیرد", metric.title
            )))
        }

        (MetricType::Text, MeasurementInput::Text(t)) => {
            // presence-based: 100 if all required keywords appear, else 0
            let t_low = t.to_lowercase();
            let all_present = metric.required_keywords.iter()
                .all(|k| t_low.contains(&k.to_lowercase()));
            if all_present { 100.0 } else { 0.0 }
        }
        (MetricType::Text, _) => {
            return Err(AppError::BadRequest(format!(
                "متریک «{}» فقط متن می‌پذیرد", metric.title
            )))
        }

        (MetricType::Scale, MeasurementInput::Number(n)) => {
            let lo = metric.scale_min.unwrap_or(1.0);
            let hi = metric.scale_max.unwrap_or(5.0);
            if hi == lo { return Ok((50.0, false)); }
            let raw = if metric.higher_is_better {
                (n - lo) / (hi - lo)
            } else {
                (hi - n) / (hi - lo)
            };
            clamp(raw * 100.0)
        }
        (MetricType::Scale, _) => {
            return Err(AppError::BadRequest(format!(
                "متریک «{}» فقط عدد می‌پذیرد", metric.title
            )))
        }
    };
    let failed = metric.critical && s < 60.0;
    Ok((s, failed))
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

    // =================== Users ===================

    pub async fn list_users(&self) -> AppResult<Vec<User>> {
        self.store.list_users_full().await
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> AppResult<User> {
        let username = req.username.trim();
        let password = req.password;
        if username.is_empty() || username.len() < 3 {
            return Err(AppError::Validation("نام کاربری باید حداقل ۳ کاراکتر باشد".into()));
        }
        if password.len() < 4 {
            return Err(AppError::Validation("رمز عبور باید حداقل ۴ کاراکتر باشد".into()));
        }
        if self.store.user_exists(username).await? {
            return Err(AppError::Validation("این نام کاربری قبلاً ثبت شده".into()));
        }
        let user = User {
            username: username.to_string(),
            password_hash: crate::auth::hash_password(&password).map_err(|e| AppError::Internal(e.to_string()))?,
            is_admin: req.is_admin.unwrap_or(false),
            created_at: Utc::now(),
        };
        self.store.put_user(&user).await?;
        Ok(user)
    }

    pub async fn update_user(&self, username: &str, req: UpdateUserRequest) -> AppResult<User> {
        let mut u = self.store.get_user(username).await?
            .ok_or_else(|| AppError::NotFound("کاربر یافت نشد".into()))?;
        if let Some(p) = req.password {
            if p.len() < 4 { return Err(AppError::Validation("رمز عبور باید حداقل ۴ کاراکتر باشد".into())); }
            u.password_hash = crate::auth::hash_password(&p).map_err(|e| AppError::Internal(e.to_string()))?;
        }
        if let Some(admin) = req.is_admin {
            u.is_admin = admin;
        }
        self.store.put_user(&u).await?;
        Ok(u)
    }

    pub async fn delete_user(&self, username: &str) -> AppResult<()> {
        // جلوگیری از حذف آخرین ادمین
        let users = self.store.list_users_full().await?;
        let target = users.iter().find(|u| u.username == username)
            .ok_or_else(|| AppError::NotFound("کاربر یافت نشد".into()))?;
        if target.is_admin {
            let admin_count = users.iter().filter(|u| u.is_admin).count();
            if admin_count <= 1 {
                return Err(AppError::Validation("آخرین مدیر سیستم قابل حذف نیست".into()));
            }
        }
        self.store.delete_user(username).await
    }

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
        // Validate every metric_id exists
        for c in &req.criteria {
            if self.store.get_metric(&c.metric_id).await?.is_none() {
                return Err(AppError::Validation(format!(
                    "متریک با شناسه «{}» یافت نشد. ابتدا آن را در تنظیمات متریک‌ها تعریف کنید",
                    c.metric_id
                )));
            }
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
        if req.measurements.len() != rubric.criteria.len() {
            return Err(AppError::Validation(format!(
                "تعداد اندازه‌گیری‌ها باید {} باشد (به تعداد معیارهای استاندارد)",
                rubric.criteria.len()
            )));
        }
        // Each criterion is a reference to a MetricDefinition. Look it up
        // and run the measurement through the metric engine.
        let mut total = 0.0_f64;
        let mut critical = false;
        let mut reasons = Vec::new();
        let mut dimension_scores = Vec::with_capacity(rubric.criteria.len());
        for (i, c) in rubric.criteria.iter().enumerate() {
            let metric = self
                .store
                .get_metric(&c.metric_id)
                .await?
                .ok_or_else(|| AppError::Validation(format!(
                    "متریک {} یافت نشد (نیاز به تعریف در تنظیمات متریک‌ها)", c.metric_id
                )))?;
            let (s, failed) = evaluate_metric(&metric, &req.measurements[i])?;
            total += s * c.weight / 100.0;
            if failed {
                critical = true;
                reasons.push(format!("{}: نمره محاسبه‌شده {} کمتر از حد بحرانی ۶۰", metric.title, s));
            }
            dimension_scores.push(s);
        }
        let overall = (total * 100.0).round() / 100.0;
        let level = level_of(overall, critical);
        let score = Score {
            id: Uuid::new_v4().to_string(),
            interaction_id: req.interaction_id.clone(),
            rubric_id: rubric.id.clone(),
            overall_score: overall,
            level,
            dimension_scores,
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

    pub async fn compute_recommendations(&self) -> AppResult<Vec<Recommendation>> {
        // Predictive QA queue: rank unscored interactions by risk.
        //   risk = w_age * age_hours
        //        + w_vip * (customer is VIP ? 1 : 0)
        //        + w_complex * (channel is chat/email/branch ? 1 : 0)
        //        + w_negative * (transcript contains negative keywords ? 1 : 0)
        //        + w_agent_low * (agent's average score is below 60 ? 1 : 0)
        let (interactions, scores, customers, agents) = tokio::try_join!(
            self.store.list_interactions(),
            self.store.scan_scores(),
            self.store.list_customers(),
            self.store.list_agents(),
        )?;

        // Pre-compute per-agent avg score for the agent_low signal
        let mut agent_scores: std::collections::HashMap<&str, Vec<f64>> = std::collections::HashMap::new();
        for s in &scores {
            // find agent via interaction (one lookup)
            if let Some(it) = interactions.iter().find(|i| i.id == s.interaction_id) {
                agent_scores.entry(&it.agent_id).or_default().push(s.overall_score);
            }
        }
        let mut agent_low: std::collections::HashMap<&str, f64> = std::collections::HashMap::new();
        for (id, list) in &agent_scores {
            let avg: f64 = list.iter().sum::<f64>() / list.len() as f64;
            if avg < 60.0 { agent_low.insert(id, avg); }
        }

        const W_AGE: f64 = 0.30;
        const W_VIP: f64 = 25.0;
        const W_COMPLEX: f64 = 15.0;
        const W_NEGATIVE: f64 = 25.0;
        const W_AGENT_LOW: f64 = 30.0;

        let neg_kw = ["شکایت", "ناراضی", "عصبانی", "لغو", "برگشت", "نارضایتی", "ناراحت", "عصبانیت", "مشکل جدی", "بد"];
        let complex = ["چت", "ایمیل", "حضوری"];

        let mut out = Vec::new();
        for i in &interactions {
            if scores.iter().any(|s| s.interaction_id == i.id) { continue; }
            let age_hours = (Utc::now() - i.created_at).num_minutes() as f64 / 60.0;

            let mut risk = 0.0_f64;
            let mut reasons = Vec::new();

            // age: linear 0..1 over 48 hours
            let age_w = W_AGE * (age_hours / 48.0).min(1.0);
            risk += age_w;
            if age_hours >= 24.0 {
                reasons.push(format!("بیش از {} ساعت از ثبت گذشته", age_hours as i64));
            }

            let customer = customers.iter().find(|c| c.id == i.customer_id);
            if let Some(c) = customer {
                if c.segment == "VIP" || c.segment == "شرکتی" {
                    risk += W_VIP;
                    reasons.push(format!("مشتری {} (اولویت بالا)", c.segment));
                }
            }

            if complex.contains(&i.channel.as_str()) {
                risk += W_COMPLEX;
                reasons.push(format!("کانال پیچیده: {}", i.channel));
            }

            let tr_low = i.transcript.to_lowercase();
            if neg_kw.iter().any(|k| tr_low.contains(&k.to_lowercase())) {
                risk += W_NEGATIVE;
                reasons.push("متن مکالمه حاوی کلمات منفی".into());
            }

            if let Some(avg) = agent_low.get(i.agent_id.as_str()) {
                risk += W_AGENT_LOW;
                reasons.push(format!("میانگین امتیاز کارشناس پایین ({:.0})", avg));
            }

            let priority = if risk >= 60.0 { "بالا" }
                           else if risk >= 30.0 { "متوسط" }
                           else { "کم" };

            let agent = agents.iter().find(|a| a.id == i.agent_id);
            out.push(Recommendation {
                interaction_id: i.id.clone(),
                agent_id: i.agent_id.clone(),
                agent_name: agent.map(|a| a.name.clone()).unwrap_or_default(),
                customer_id: i.customer_id.clone(),
                customer_name: customer.map(|c| c.name.clone()).unwrap_or_default(),
                channel: i.channel.clone(),
                subject: i.subject.clone(),
                priority: priority.into(),
                risk_score: (risk * 10.0).round() / 10.0,
                reasons,
                suggested_action: "ارزیابی تعامل طبق استاندارد مربوطه".into(),
                age_hours: (age_hours * 10.0).round() / 10.0,
            });
        }
        out.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(out)
    }

    // =================== Metric Definitions ===================

    pub async fn list_metrics(&self) -> AppResult<Vec<MetricDefinition>> {
        self.store.list_metrics().await
    }

    pub async fn create_metric(&self, req: CreateMetricRequest) -> AppResult<MetricDefinition> {
        if req.title.trim().is_empty() { return Err(AppError::Validation("عنوان متریک الزامی است".into())); }
        if req.code.trim().is_empty() { return Err(AppError::Validation("کد متریک الزامی است".into())); }
        if let Some(existing) = self.store.get_metric_by_code(&req.code).await? {
            return Err(AppError::Conflict(format!("کد متریک {} تکراری است", existing.code)));
        }
        match req.metric_type {
            MetricType::Numeric => {
                let lo = req.min.unwrap_or(0.0);
                let hi = req.max.unwrap_or(100.0);
                if hi <= lo { return Err(AppError::Validation("برای نوع عددی، max باید بزرگتر از min باشد".into())); }
            }
            MetricType::Categorical => {
                if req.allowed_values.is_empty() {
                    return Err(AppError::Validation("برای نوع دسته‌ای، مقادیر مجاز الزامی است".into()));
                }
            }
            MetricType::Text => {
                if req.required_keywords.is_empty() {
                    return Err(AppError::Validation("برای نوع متنی، کلیدواژه‌های الزامی را وارد کنید".into()));
                }
            }
            MetricType::Scale => {
                let lo = req.scale_min.unwrap_or(1.0);
                let hi = req.scale_max.unwrap_or(5.0);
                if hi <= lo { return Err(AppError::Validation("برای نوع مقیاس، scale_max باید بزرگتر از scale_min باشد".into())); }
            }
            MetricType::Boolean => {}
        }
        let m = MetricDefinition {
            id: Uuid::new_v4().to_string(),
            code: req.code,
            title: req.title,
            description: req.description,
            category: req.category,
            metric_type: req.metric_type,
            min: req.min,
            max: req.max,
            higher_is_better: req.higher_is_better,
            allowed_values: req.allowed_values,
            value_scores: req.value_scores,
            required_keywords: req.required_keywords,
            scale_min: req.scale_min,
            scale_max: req.scale_max,
            critical: req.critical,
            created_at: Utc::now(),
        };
        self.store.put_metric(&m).await?;
        Ok(m)
    }

    pub async fn update_metric(&self, id: &str, req: UpdateMetricRequest) -> AppResult<MetricDefinition> {
        let mut m = self.store.get_metric(id).await?
            .ok_or_else(|| AppError::NotFound("متریک یافت نشد".into()))?;
        if let Some(t) = req.title { m.title = t; }
        if let Some(d) = req.description { m.description = d; }
        if let Some(c) = req.category { m.category = c; }
        if let Some(v) = req.min { m.min = Some(v); }
        if let Some(v) = req.max { m.max = Some(v); }
        if let Some(v) = req.higher_is_better { m.higher_is_better = v; }
        if let Some(v) = req.allowed_values { m.allowed_values = v; }
        if let Some(v) = req.value_scores { m.value_scores = v; }
        if let Some(v) = req.required_keywords { m.required_keywords = v; }
        if let Some(v) = req.scale_min { m.scale_min = Some(v); }
        if let Some(v) = req.scale_max { m.scale_max = Some(v); }
        if let Some(v) = req.critical { m.critical = v; }
        self.store.put_metric(&m).await?;
        Ok(m)
    }

    pub async fn delete_metric(&self, id: &str) -> AppResult<()> {
        // refuse to delete if any rubric references it
        let rubrics = self.store.list_rubrics().await?;
        for r in rubrics {
            if r.criteria.iter().any(|c| c.metric_id == id) {
                return Err(AppError::Conflict(format!(
                    "این متریک در استاندارد «{}» استفاده شده و قابل حذف نیست", r.name
                )));
            }
        }
        self.store.delete_metric(id).await
    }

    // =================== Dashboard ===================

    pub async fn dashboard(&self) -> AppResult<serde_json::Value> {
        let (agents, customers, interactions, scores, issues) = tokio::try_join!(
            self.store.list_agents(),
            self.store.list_customers(),
            self.store.list_interactions(),
            self.store.scan_scores(),
            self.store.list_issues(),
        )?;

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
