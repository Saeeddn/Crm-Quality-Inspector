//! Unit tests for the CRM Quality Inspector.
//!
//! These tests do NOT require a running database or any external service.
//! They cover pure-logic concerns: model serde round-trips, session
//! management, and password hashing.
//!
//! Run with:
//!     cargo test --test unit_test

use crm_qi::auth::{hash_password, verify_password, SessionStore};
use crm_qi::models::{
    Agent, CreateAgentRequest, CreateCustomerRequest, CreateInteractionRequest, Customer,
    Interaction, Issue, ListQuery, MeasurementInput, Rubric, RubricCriterion, Score, ScoreRequest,
    User,
};

// -------------------- Model serde round-trips --------------------

#[test]
fn agent_struct_serde_roundtrip() {
    let a = Agent {
        id: "ag-1".into(),
        name: "علی رضایی".into(),
        department: "بانکداری".into(),
        position: "کارشناس ارشد".into(),
        active: true,
        created_at: chrono::Utc::now(),
    };
    let s = serde_json::to_string(&a).unwrap();
    let back: Agent = serde_json::from_str(&s).unwrap();
    assert_eq!(back.id, "ag-1");
    assert_eq!(back.name, "علی رضایی");
    assert_eq!(back.department, "بانکداری");
    assert!(back.active);
}

#[test]
fn customer_struct_serde_roundtrip() {
    let c = Customer {
        id: "cu-1".into(),
        name: "شرکت پارس".into(),
        phone: "021-12345678".into(),
        product_type: "حساب".into(),
        segment: "ویژه".into(),
        notes: "VIP".into(),
        created_at: chrono::Utc::now(),
    };
    let s = serde_json::to_string(&c).unwrap();
    let back: Customer = serde_json::from_str(&s).unwrap();
    assert_eq!(back.id, "cu-1");
    assert_eq!(back.product_type, "حساب");
    assert_eq!(back.segment, "ویژه");
}

#[test]
fn interaction_struct_serde_roundtrip() {
    let i = Interaction {
        id: "in-1".into(),
        agent_id: "ag-1".into(),
        customer_id: "cu-1".into(),
        channel: "phone".into(),
        subject: "مشاوره افتتاح حساب".into(),
        transcript: "متن مکالمه...".into(),
        tags: vec!["مهم".to_string(), "VIP".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let s = serde_json::to_string(&i).unwrap();
    let back: Interaction = serde_json::from_str(&s).unwrap();
    assert_eq!(back.id, "in-1");
    assert_eq!(back.channel, "phone");
    assert_eq!(back.tags, vec!["مهم", "VIP"]);
}

#[test]
fn rubric_criterion_serde_preserves_critical_flag() {
    let c = RubricCriterion {
        metric_id: "CMP-compliance".into(),
        weight: 25.0,
        critical: true,
    };
    let s = serde_json::to_string(&c).unwrap();
    let back: RubricCriterion = serde_json::from_str(&s).unwrap();
    assert!(back.critical);
    assert_eq!(back.weight, 25.0);
    assert_eq!(back.metric_id, "CMP-compliance");
}

#[test]
fn rubric_full_serde_roundtrip() {
    let r = Rubric {
        id: "rb-1".into(),
        name: "چک‌لیست بانکداری".into(),
        department: "بانکداری".into(),
        product_type: Some("حساب".into()),
        channel: Some("phone".into()),
        version: 2,
        criteria: vec![
            RubricCriterion { metric_id: "Greeting".into(), weight: 10.0, critical: false },
            RubricCriterion { metric_id: "Compliance".into(), weight: 30.0, critical: true },
        ],
        active: true,
        created_at: chrono::Utc::now(),
    };
    let s = serde_json::to_string(&r).unwrap();
    let back: Rubric = serde_json::from_str(&s).unwrap();
    assert_eq!(back.id, "rb-1");
    assert_eq!(back.version, 2);
    assert_eq!(back.criteria.len(), 2);
    assert!(back.criteria[1].critical);
}

#[test]
fn score_struct_serde_roundtrip() {
    let sc = Score {
        id: "sc-1".into(),
        interaction_id: "in-1".into(),
        rubric_id: "rb-1".into(),
        overall_score: 87.5,
        level: "عالی".into(),
        dimension_scores: vec![80.0, 90.0, 85.0],
        critical_fail: false,
        critical_fail_reasons: vec![],
        evaluator: "qa-supervisor".into(),
        notes: "عملکرد خوب".into(),
        created_at: chrono::Utc::now(),
    };
    let s = serde_json::to_string(&sc).unwrap();
    let back: Score = serde_json::from_str(&s).unwrap();
    assert_eq!(back.overall_score, 87.5);
    assert_eq!(back.level, "عالی");
    assert_eq!(back.dimension_scores.len(), 3);
    assert!(!back.critical_fail);
}

#[test]
fn score_request_serde_roundtrip() {
    let req = ScoreRequest {
        interaction_id: "in-1".into(),
        rubric_id: Some("rb-1".into()),
        measurements: vec![
            MeasurementInput::Number(95.0),
            MeasurementInput::Number(80.0),
        ],
        evaluator: Some("qa".into()),
        notes: "ok".into(),
    };
    let s = serde_json::to_string(&req).unwrap();
    let back: ScoreRequest = serde_json::from_str(&s).unwrap();
    assert_eq!(back.interaction_id, "in-1");
    assert_eq!(back.measurements.len(), 2);
    match &back.measurements[0] {
        MeasurementInput::Number(v) => assert!((v - 95.0).abs() < 0.001),
        _ => panic!("expected Number variant"),
    }
    assert_eq!(back.evaluator.as_deref(), Some("qa"));
}

#[test]
fn score_request_supports_bool_measurements() {
    let req = ScoreRequest {
        interaction_id: "in-2".into(),
        rubric_id: None,
        measurements: vec![MeasurementInput::Bool(true), MeasurementInput::Bool(false)],
        evaluator: None,
        notes: "".into(),
    };
    let s = serde_json::to_string(&req).unwrap();
    let back: ScoreRequest = serde_json::from_str(&s).unwrap();
    assert_eq!(back.measurements.len(), 2);
    assert!(matches!(back.measurements[0], MeasurementInput::Bool(true)));
    assert!(matches!(back.measurements[1], MeasurementInput::Bool(false)));
    assert!(back.rubric_id.is_none());
}

#[test]
fn issue_struct_serde_roundtrip() {
    let i = Issue {
        id: "is-1".into(),
        interaction_id: "in-1".into(),
        agent_id: "ag-1".into(),
        severity: "high".into(),
        category: "compliance".into(),
        description: "عدم احراز هویت".into(),
        status: "open".into(),
        root_cause: None,
        corrective_action: None,
        due_at: None,
        created_at: chrono::Utc::now(),
        resolved_at: None,
    };
    let s = serde_json::to_string(&i).unwrap();
    let back: Issue = serde_json::from_str(&s).unwrap();
    assert_eq!(back.id, "is-1");
    assert_eq!(back.severity, "high");
    assert_eq!(back.status, "open");
    assert_eq!(back.category, "compliance");
    assert!(back.root_cause.is_none());
    assert!(back.resolved_at.is_none());
}

#[test]
fn user_struct_roundtrip() {
    let u = User {
        username: "bob".into(),
        password_hash: "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8".into(),
        is_admin: false,
        created_at: chrono::Utc::now(),
    };
    let s = serde_json::to_string(&u).unwrap();
    let back: User = serde_json::from_str(&s).unwrap();
    assert_eq!(back.username, "bob");
    assert!(!back.is_admin);
}

#[test]
fn list_query_defaults() {
    let q = ListQuery::default();
    assert!(q.department.is_none());
    assert!(q.limit.is_none());
    let json = serde_json::to_string(&q).unwrap();
    let back: ListQuery = serde_json::from_str(&json).unwrap();
    assert!(back.department.is_none());
    assert!(back.limit.is_none());
}

#[test]
fn list_query_accepts_any_limit_value() {
    // ListQuery stores the raw limit — clamping happens in the handler/store.
    // This test just ensures deserialization works for large values.
    let q: ListQuery = serde_json::from_str(r#"{"limit": 99999}"#).unwrap();
    assert_eq!(q.limit, Some(99999));
}

// -------------------- Create request DTOs --------------------

#[test]
fn create_agent_request_requires_all_fields() {
    // CreateAgentRequest has no #[serde(default)] — all fields are required
    let json = r#"{"name":"سارا","department":"فروش","position":"کارشناس"}"#;
    let req: CreateAgentRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.name, "سارا");
    assert_eq!(req.position, "کارشناس");
}

#[test]
fn create_agent_request_rejects_missing_fields() {
    let json = r#"{"name":"سارا"}"#;
    let result: Result<CreateAgentRequest, _> = serde_json::from_str(json);
    assert!(result.is_err(), "missing department and position should fail");
}

#[test]
fn create_customer_request_defaults() {
    let json = r#"{"name":"مشتری تست"}"#;
    let req: CreateCustomerRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.name, "مشتری تست");
    assert_eq!(req.phone, "");
    assert_eq!(req.product_type, "");
    assert_eq!(req.segment, "");
}

#[test]
fn create_interaction_request_requires_all_fields() {
    let json = r#"{"agent_id":"a1","customer_id":"c1","channel":"phone","subject":"test","transcript":"hello"}"#;
    let req: CreateInteractionRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.transcript, "hello");
    assert!(req.tags.is_empty());
}

#[test]
fn create_interaction_request_rejects_missing_transcript() {
    let json = r#"{"agent_id":"a1","customer_id":"c1","channel":"phone","subject":"test"}"#;
    let result: Result<CreateInteractionRequest, _> = serde_json::from_str(json);
    assert!(result.is_err(), "missing transcript should fail");
}

// -------------------- Session token format --------------------

#[test]
fn session_token_is_64_hex_chars() {
    let store = SessionStore::new();
    let s = store.create("alice", true);
    // Token should be 64 hex chars (32 bytes from random source)
    assert_eq!(s.token.len(), 64, "token must be 64 hex chars");
    assert!(s.token.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn session_validate_returns_correct_user() {
    let store = SessionStore::new();
    let s = store.create("diana", false);
    let restored = store.validate(&s.token).expect("token should validate");
    assert_eq!(restored.username, "diana");
    assert!(!restored.is_admin);
}

#[test]
fn session_invalid_token_returns_none() {
    let store = SessionStore::new();
    assert!(store.validate("not-a-real-token").is_none());
    assert!(store.validate("").is_none());
}

#[test]
fn session_create_assigns_ttl_in_future() {
    let store = SessionStore::new();
    let s = store.create("ttl-test", true);
    assert!(s.expires_at > chrono::Utc::now());
}

// -------------------- Password hashing --------------------

#[test]
fn password_hash_and_verify_roundtrip() {
    let pw = "MyStr0ng!Pass2026";
    let hash = hash_password(pw).expect("hash should succeed");
    assert!(!hash.is_empty());
    assert!(verify_password(pw, &hash).unwrap());
    assert!(!verify_password("wrong-password", &hash).unwrap());
}

#[test]
fn password_hash_is_deterministic() {
    // SHA-256 (current implementation) is deterministic — same input → same hash.
    // This is documented behavior; if we ever switch to bcrypt/argon2 this test
    // should be inverted.
    let pw = "SamePassword123!";
    let h1 = hash_password(pw).unwrap();
    let h2 = hash_password(pw).unwrap();
    assert_eq!(h1, h2, "SHA-256 hashes must be deterministic for the same input");
}
