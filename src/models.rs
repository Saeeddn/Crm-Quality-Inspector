use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub code: String,
    pub title: String,
    pub description: String,
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
    pub scores: Vec<f64>,
    pub evaluator: Option<String>,
    #[serde(default)]
    pub notes: String,
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
}
