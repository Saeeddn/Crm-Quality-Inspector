use crm_qi::auth::SessionStore;
use crm_qi::models::*;

#[test]
fn session_create_and_validate_roundtrip() {
    let store = SessionStore::new();
    let session = store.create("alice", true);
    assert!(!session.token.is_empty());
    let restored = store.validate(&session.token);
    assert!(restored.is_some());
    let s = restored.unwrap();
    assert_eq!(s.username, "alice");
    assert!(s.is_admin);
}

#[test]
fn session_invalid_token_returns_none() {
    let store = SessionStore::new();
    assert!(store.validate("does-not-exist").is_none());
}

#[test]
fn score_request_serializes_correctly() {
    let req = ScoreRequest {
        interaction_id: "in-1".into(),
        rubric_id: Some("rb-1".into()),
        scores: vec![80.0, 90.0],
        evaluator: Some("qa".into()),
        notes: "ok".into(),
    };
    let s = serde_json::to_string(&req).unwrap();
    let back: ScoreRequest = serde_json::from_str(&s).unwrap();
    assert_eq!(back.interaction_id, "in-1");
    assert_eq!(back.scores.len(), 2);
    assert_eq!(back.evaluator.as_deref(), Some("qa"));
}

#[test]
fn rubric_criterion_serde_preserves_critical_flag() {
    let c = RubricCriterion {
        code: "CMP".into(),
        title: "انطباق".into(),
        description: "تست".into(),
        weight: 20.0,
        critical: true,
    };
    let s = serde_json::to_string(&c).unwrap();
    let back: RubricCriterion = serde_json::from_str(&s).unwrap();
    assert!(back.critical);
    assert_eq!(back.weight, 20.0);
}

#[test]
fn user_struct_roundtrip() {
    let u = User {
        username: "bob".into(),
        password_hash: "$2b$12$abcdef".into(),
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
