use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// =====================
// KPI (شاخص قابل اندازه‌گیری برای امتیازدهی خودکار)
// =====================

/// نوع شاخص قابل اندازه‌گیری. هر نوع یک استراتژی اندازه‌گیری دارد.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KpiKind {
    /// تعداد وقوع یک کلمه/عبارت در متن (مثل: "تعداد احوالپرسی")
    KeywordCount,
    /// وجود/عدم وجود یک کلمه یا عبارت (باینری)
    KeywordPresence,
    /// تعداد کاراکتر یا کلمه در متن
    TextLength,
    /// نسبت یک کلمه به کل (مثل: نسبت کلمات منفی)
    KeywordRatio,
    /// زمان پاسخگویی (میلی‌ثانیه) — عددی
    ResponseTime,
    /// امتیاز دستی بازه‌ای (برای مواردی که خودکار ممکن نیست، مثل: لحن)
    ManualRange,
}

/// یک معیار قابل اندازه‌گیری برای امتیازدهی خودکار
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Kpi {
    pub id: String,
    pub code: String,             // شناسه کوتاه (مثل greeting_count)
    pub name: String,             // نام نمایشی فارسی
    pub kind: KpiKind,
    pub description: String,
    /// برای KeywordCount/KeywordPresence/KeywordRatio: الگوی جستجو
    pub pattern: Option<String>,
    /// برای KeywordCount/Ratio: آستانه مورد انتظار (پیش‌فرض)
    pub threshold: Option<f64>,
    /// برای KeywordRatio: کلیدواژه شمارنده (مثل "کلمات منفی")
    pub ratio_total_pattern: Option<String>,
    /// وزن این KPI در محاسبه نمره کل criterion (0-100)
    pub weight: f64,
    /// آیا بحرانی است؟ (اگر نمره < 60، کل interaction شکست بحرانی)
    pub critical: bool,
    /// وضعیت فعال
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

/// درخواست ایجاد KPI جدید
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KpiCreateReq {
    pub code: String,
    pub name: String,
    pub kind: KpiKind,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub threshold: Option<f64>,
    #[serde(default)]
    pub ratio_total_pattern: Option<String>,
    pub weight: f64,
    #[serde(default)]
    pub critical: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToggleKpiRequest { pub active: bool }

/// نتیجه اندازه‌گیری یک KPI روی یک transcript
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KpiMeasurement {
    pub kpi_id: String,
    pub kpi_code: String,
    pub kpi_name: String,
    pub kind: KpiKind,
    pub measured: f64,        // مقدار اندازه‌گیری شده
    pub expected: Option<f64>, // مقدار مورد انتظار
    pub score: f64,           // 0-100
    pub weight: f64,
    pub weighted: f64,        // score * weight / 100
    pub critical: bool,
    pub critical_fail: bool,
    pub evidence: String,     // توضیح فارسی کوتاه: "یافت شد ۳ بار"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskFactor {
    pub code: String,         // missing_score, vip_customer, low_agent_avg, complex_channel, sentiment_negative
    pub label: String,        // نام فارسی
    pub points: f64,          // امتیاز ریسک
    pub reason: String,       // دلیل فارسی
}

/// Per-customer health/risk score. Distinct from `QaRecommendation` which is
/// per-interaction. A customer's risk is an aggregate of all interactions,
/// open issues, agent performance, and recency.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerRiskScore {
    pub customer_id: String,
    pub customer_name: String,
    pub risk_score: f64,         // 0-100
    pub level: String,           // "بالا" / "متوسط" / "پایین"
    pub factors: Vec<RiskFactor>,
    pub total_interactions: u32,
    pub scored_interactions: u32,
    pub open_issues: u32,
    pub avg_score: Option<f64>,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub recommended_action: String,  // "تماس فوری" / "پیگیری" / "نظارت"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QaRecommendation {
    pub interaction_id: String,
    pub subject: String,
    pub agent_id: String,
    pub agent_name: String,
    pub customer_id: String,
    pub customer_name: String,
    pub channel: String,
    pub created_at: DateTime<Utc>,
    pub risk_score: f64,      // 0-100
    pub priority: String,     // "بالا" / "متوسط" / "پایین"
    pub reason: String,
    pub factors: Vec<RiskFactor>,
    pub suggested_action: String,
}

// =====================
// User & Auth
// =====================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub username: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub is_admin: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub password: Option<String>,
    pub is_admin: Option<bool>,
}

// =====================
// Agent (نماینده / کارشناس)
// =====================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub department: String,
    pub position: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub department: String,
    pub position: String,
}

#[derive(Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub active: Option<bool>,
}

// =====================
// Customer (مشتری)
// =====================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub product_type: String,
    pub segment: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    pub name: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub product_type: String,
    #[serde(default)]
    pub segment: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Deserialize)]
pub struct UpdateCustomerRequest {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub product_type: Option<String>,
    pub segment: Option<String>,
    pub notes: Option<String>,
}

// =====================
// Interaction (تعامل)
// =====================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interaction {
    pub id: String,
    pub agent_id: String,
    pub customer_id: String,
    pub channel: String,
    pub subject: String,
    pub transcript: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateInteractionRequest {
    pub agent_id: String,
    pub customer_id: String,
    pub channel: String,
    pub subject: String,
    pub transcript: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

// =====================
// Rubric (استاندارد ارزیابی)
// =====================

/// A rubric criterion is just a reference to a MetricDefinition,
/// plus its weight within this rubric and whether it's critical.
/// The actual measurement (title/description/measurement logic)
/// is owned by the MetricDefinition and cannot be edited inline.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub metric_id: String,
    pub weight: f64,
    pub critical: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rubric {
    pub id: String,
    pub name: String,
    pub department: String,
    pub product_type: Option<String>,
    pub channel: Option<String>,
    pub version: u32,
    pub criteria: Vec<RubricCriterion>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateRubricRequest {
    pub name: String,
    #[serde(default)]
    pub department: String,
    pub product_type: Option<String>,
    pub channel: Option<String>,
    pub criteria: Vec<RubricCriterion>,
}

// =====================
// Score (ارزیابی)
// =====================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Score {
    pub id: String,
    pub interaction_id: String,
    pub rubric_id: String,
    pub overall_score: f64,
    pub level: String,
    pub dimension_scores: Vec<f64>,
    pub critical_fail: bool,
    pub critical_fail_reasons: Vec<String>,
    pub evaluator: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct ScoreRequest {
    pub interaction_id: String,
    pub rubric_id: Option<String>,
    /// For each rubric criterion (in order), pass the RAW measurement
    /// value. The engine converts to a 0-100 score using the metric's
    /// measurement type and threshold config.
    pub measurements: Vec<MeasurementInput>,
    pub evaluator: Option<String>,
    #[serde(default)]
    pub notes: String,
}

/// One raw measurement value supplied by the QA evaluator.
/// The shape depends on the metric's measurement type:
///   Boolean:   bool
///   Numeric:   f64
///   Categorical: String (must equal one of the metric's allowed values)
///   Text:      String (presence-based scoring)
///   Scale:     f64
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MeasurementInput {
    Bool(bool),
    Number(f64),
    Text(String),
}

// =====================
// Issue (ایراد / اقدام اصلاحی)
// =====================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub interaction_id: String,
    pub agent_id: String,
    pub severity: String,   // بحرانی / بالا / متوسط / پایین
    pub category: String,   // دسته‌بندی
    pub description: String,
    pub status: String,     // باز / در حال بررسی / بسته
    pub root_cause: Option<String>,
    pub corrective_action: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct ResolveIssueRequest {
    pub root_cause: String,
    pub corrective_action: String,
}

#[derive(Deserialize)]
pub struct CreateIssueRequest {
    pub interaction_id: String,
    pub agent_id: String,
    pub severity: String,   // بحرانی / بالا / متوسط / پایین
    pub category: String,
    pub description: String,
    #[serde(default)]
    pub status: String,     // باز / در حال بررسی / بسته
}

// =====================
// MetricDefinition (پارامتر اندازه‌گیری)
// =====================

/// Type of measurement. Determines how raw values are scored.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// Yes/No. score = 100 if true, 0 if false.
    Boolean,
    /// Number within [min, max]. score = linear interpolation
    /// where min maps to 0 and max maps to 100.
    Numeric,
    /// One of the allowed_values. score = value_to_score[chosen].
    Categorical,
    /// Free text. score = 100 if required_keywords ALL appear,
    /// else 0. (Absence-based for compliance.)
    Text,
    /// 1-5 or 1-10 scale. score = (value - 1) / (max - 1) * 100.
    Scale,
}

/// A single measurement parameter. Defines WHAT is being measured
/// and HOW a raw value is converted to a 0-100 score. Rubrics only
/// REFERENCE these by id; the criterion's title/description/weight
/// is shared, and inline editing of criteria is impossible.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub id: String,
    /// Short stable code, e.g. "compliance_know_your_customer".
    pub code: String,
    pub title: String,
    pub description: String,
    pub category: String, // e.g. "انطباق" / "ارتباط" / "حل مسئله"
    #[serde(rename = "type")]
    pub metric_type: MetricType,

    // For Numeric:
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    /// Direction: true (default) means higher = better,
    /// false means lower = better (e.g. complaint rate).
    #[serde(default = "default_higher_is_better")]
    pub higher_is_better: bool,

    // For Categorical:
    #[serde(default)]
    pub allowed_values: Vec<String>,
    /// value -> score. If unset, the metric engine uses default rules.
    #[serde(default)]
    pub value_scores: std::collections::HashMap<String, f64>,

    // For Text:
    #[serde(default)]
    pub required_keywords: Vec<String>,

    // For Scale:
    #[serde(default)]
    pub scale_min: Option<f64>,
    #[serde(default)]
    pub scale_max: Option<f64>,

    /// Whether failure on this metric is a critical fail.
    /// (e.g. compliance violations are always critical.)
    #[serde(default)]
    pub critical: bool,

    pub created_at: DateTime<Utc>,
}

fn default_higher_is_better() -> bool { true }

#[derive(Deserialize)]
pub struct CreateMetricRequest {
    pub code: String,
    pub title: String,
    pub description: String,
    pub category: String,
    #[serde(rename = "type")]
    pub metric_type: MetricType,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default = "default_higher_is_better")]
    pub higher_is_better: bool,
    #[serde(default)]
    pub allowed_values: Vec<String>,
    #[serde(default)]
    pub value_scores: std::collections::HashMap<String, f64>,
    #[serde(default)]
    pub required_keywords: Vec<String>,
    #[serde(default)]
    pub scale_min: Option<f64>,
    #[serde(default)]
    pub scale_max: Option<f64>,
    #[serde(default)]
    pub critical: bool,
}

#[derive(Deserialize, Default)]
pub struct UpdateMetricRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub higher_is_better: Option<bool>,
    pub allowed_values: Option<Vec<String>>,
    pub value_scores: Option<std::collections::HashMap<String, f64>>,
    pub required_keywords: Option<Vec<String>>,
    pub scale_min: Option<f64>,
    pub scale_max: Option<f64>,
    pub critical: Option<bool>,
}

// =====================
// Recommendation (پیشنهاد QA)
// =====================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recommendation {
    pub interaction_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub customer_id: String,
    pub customer_name: String,
    pub channel: String,
    pub subject: String,
    pub priority: String, // بالا / متوسط / پایین
    pub risk_score: f64,
    pub reasons: Vec<String>,
    pub suggested_action: String,
    pub age_hours: f64,
}

// =====================
// ListQuery (فیلترهای عمومی)
// =====================

#[derive(Deserialize, Serialize, Default)]
pub struct ListQuery {
    #[serde(default)]
    pub department: Option<String>,
    #[serde(default)]
    pub product_type: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub customer_id: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub page: Option<i64>,
}

// =====================
// Coaching Plan (Closed-Loop QA)
// =====================

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
        pub status: String,
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

    // =====================
    // Calibration Session
    // =====================

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum CalibrationSessionStatus {
        Draft,
        Scoring,
        InSession,
        Completed,
        Cancelled,
    }

    impl std::fmt::Display for CalibrationSessionStatus {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = format!("{:?}", self).to_lowercase();
            write!(f, "{}", s)
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CalibrationSession {
        pub id: String,
        pub name: String,
        pub status: String,
        pub rubric_id: String,
        pub reviewer_usernames: Vec<String>,
        pub sample_interaction_ids: Vec<String>,
        pub target_agreement_rate: Option<f64>,
        pub min_reviewers_per_interaction: u32,
        pub deadline_at: DateTime<Utc>,
        pub meeting_started_at: Option<DateTime<Utc>>,
        pub facilitator_id: Option<String>,
        pub agreement_rate: Option<f64>,
        pub variance_per_criterion: Option<serde_json::Value>,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CalibrationScore {
        pub id: String,
        pub session_id: String,
        pub interaction_id: String,
        pub reviewer: String,
        pub submitted_at: DateTime<Utc>,
        pub criterion_scores: serde_json::Value,
        pub overall_score: f64,
        pub notes: Option<String>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CalibrationDecision {
        pub id: String,
        pub session_id: String,
        pub criterion_id: String,
        pub agreed_interpretation: String,
        pub example_interaction_id: Option<String>,
        pub rubric_edit_proposed: Option<String>,
        pub created_at: DateTime<Utc>,
    }
