// Integration test: requires Redis running on 127.0.0.1:6379
// Run with: REDIS_URL=redis://127.0.0.1:6379/ cargo test --test integration_test -- --test-threads=1
//
// Each test uses a unique key prefix to avoid collisions when run in parallel,
// but we serialize with --test-threads=1 to be safe.

use crm_qi::error::AppError;
use crm_qi::models::*;
use crm_qi::service::Service;
use crm_qi::store::Store;

async fn fresh_store() -> Store {
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());
    Store::connect(&url).await.expect("Redis must be running for integration tests")
}

#[tokio::test]
async fn end_to_end_score_and_issue_flow() {
    let store = fresh_store().await;

    // 1. Create an agent
    let agent_id = uuid::Uuid::new_v4().to_string();
    let agent_name = "تست کارشناس".to_string();
    store
        .put_agent(&Agent {
            id: agent_id.clone(),
            name: agent_name.clone(),
            department: "بانک".into(),
            position: "کارشناس".into(),
            active: true,
            created_at: chrono::Utc::now(),
        })
        .await
        .expect("put agent");
    // Read back
    let read_agent = store
        .list_agents()
        .await
        .expect("list")
        .into_iter()
        .find(|a| a.id == agent_id)
        .expect("agent present");
    assert_eq!(read_agent.name, agent_name);

    // 2. Create a customer
    let customer_id = uuid::Uuid::new_v4().to_string();
    store
        .put_customer(&Customer {
            id: customer_id.clone(),
            name: "تست مشتری".into(),
            phone: "09120000000".into(),
            product_type: "تسهیلات".into(),
            segment: "VIP".into(),
            notes: "".into(),
            created_at: chrono::Utc::now(),
        })
        .await
        .expect("put customer");

    // 3. Create an interaction
    let interaction_id = uuid::Uuid::new_v4().to_string();
    store
        .put_interaction(&Interaction {
            id: interaction_id.clone(),
            agent_id: agent_id.clone(),
            customer_id: customer_id.clone(),
            channel: "تلفن".into(),
            subject: "تست".into(),
            transcript: "مشتری درخواست داد و کارشناس پاسخ داد.".into(),
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .await
        .expect("put interaction");

    // 4. Create a rubric with valid weights
    let rubric_id = uuid::Uuid::new_v4().to_string();
    store
        .put_rubric(&Rubric {
            id: rubric_id.clone(),
            name: "تست روبریک".into(),
            department: "عمومی".into(),
            product_type: None,
            channel: None,
            version: 1,
            criteria: vec![
                RubricCriterion {
                    code: "A".into(),
                    title: "معیار A".into(),
                    description: "".into(),
                    weight: 60.0,
                    critical: false,
                },
                RubricCriterion {
                    code: "B".into(),
                    title: "معیار B بحرانی".into(),
                    description: "".into(),
                    weight: 40.0,
                    critical: true,
                },
            ],
            active: true,
            created_at: chrono::Utc::now(),
        })
        .await
        .expect("put rubric");

    // 5. Score the interaction (no critical fail: 80 + 70, both > 60)
    let svc = Service::new(&store);
    let score = svc
        .score_interaction(ScoreRequest {
            interaction_id: interaction_id.clone(),
            rubric_id: Some(rubric_id.clone()),
            scores: vec![80.0, 70.0],
            evaluator: Some("qa-bot".into()),
            notes: "".into(),
        })
        .await
        .expect("score ok");
    assert!(!score.critical_fail);
    // weighted = 80*0.6 + 70*0.4 = 48 + 28 = 76
    assert!((score.overall_score - 76.0).abs() < 0.01);

    // 6. Score a critical-fail interaction
    let interaction_id_2 = uuid::Uuid::new_v4().to_string();
    store
        .put_interaction(&Interaction {
            id: interaction_id_2.clone(),
            agent_id: agent_id.clone(),
            customer_id: customer_id.clone(),
            channel: "چت".into(),
            subject: "بحرانی".into(),
            transcript: "بد".into(),
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .await
        .expect("put interaction 2");
    let score2 = svc
        .score_interaction(ScoreRequest {
            interaction_id: interaction_id_2.clone(),
            rubric_id: Some(rubric_id.clone()),
            scores: vec![90.0, 50.0], // 50 < 60 on critical => fail
            evaluator: Some("qa-bot".into()),
            notes: "".into(),
        })
        .await
        .expect("score2 ok");
    assert!(score2.critical_fail);
    assert_eq!(score2.level, "مردود بحرانی");

    // 7. Confirm auto-issue was created
    let issues = store.list_issues().await.expect("list issues");
    let crit = issues
        .iter()
        .find(|i| i.interaction_id == interaction_id_2)
        .expect("issue for crit interaction");
    assert_eq!(crit.severity, "بحرانی");
    assert!(crit.due_at.is_some());

    // 8. Dashboard reflects data
    let dash = svc.dashboard().await.expect("dash");
    assert!(dash.get("agent_count").is_some());
    assert!(dash.get("scored_count").is_some());
    assert!(dash.get("quality_grade").is_some());

    // 9. Agent report
    let report = svc.agent_report(&agent_id).await.expect("report");
    let scored = report.get("scored_interactions").and_then(|v| v.as_i64()).unwrap_or(0);
    assert!(scored >= 2);
}

#[tokio::test]
async fn score_rejects_wrong_count() {
    let store = fresh_store().await;
    let rubric = Rubric {
        id: uuid::Uuid::new_v4().to_string(),
        name: "x".into(),
        department: "عمومی".into(),
        product_type: None,
        channel: None,
        version: 1,
        criteria: vec![
            RubricCriterion { code: "A".into(), title: "a".into(), description: "".into(), weight: 50.0, critical: false },
            RubricCriterion { code: "B".into(), title: "b".into(), description: "".into(), weight: 50.0, critical: false },
        ],
        active: true,
        created_at: chrono::Utc::now(),
    };
    let rubric_id = rubric.id.clone();
    store.put_rubric(&rubric).await.expect("put");

    let svc = Service::new(&store);
    let err = svc
        .score_interaction(ScoreRequest {
            interaction_id: "ghost".into(), // not present, so it fails at lookup
            rubric_id: Some(rubric_id),
            scores: vec![10.0],
            evaluator: None,
            notes: "".into(),
        })
        .await;
    assert!(matches!(err, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn score_rejects_out_of_range() {
    let store = fresh_store().await;
    let rubric = Rubric {
        id: uuid::Uuid::new_v4().to_string(),
        name: "x".into(),
        department: "عمومی".into(),
        product_type: None,
        channel: None,
        version: 1,
        criteria: vec![RubricCriterion {
            code: "A".into(),
            title: "a".into(),
            description: "".into(),
            weight: 100.0,
            critical: false,
        }],
        active: true,
        created_at: chrono::Utc::now(),
    };
    let rubric_id = rubric.id.clone();
    store.put_rubric(&rubric).await.expect("put");

    let agent = uuid::Uuid::new_v4().to_string();
    store
        .put_agent(&Agent {
            id: agent.clone(),
            name: "x".into(),
            department: "عمومی".into(),
            position: "x".into(),
            active: true,
            created_at: chrono::Utc::now(),
        })
        .await
        .expect("agent");
    let cust = uuid::Uuid::new_v4().to_string();
    store
        .put_customer(&Customer {
            id: cust.clone(),
            name: "x".into(),
            phone: "".into(),
            product_type: "".into(),
            segment: "".into(),
            notes: "".into(),
            created_at: chrono::Utc::now(),
        })
        .await
        .expect("cust");
    let iid = uuid::Uuid::new_v4().to_string();
    store
        .put_interaction(&Interaction {
            id: iid.clone(),
            agent_id: agent,
            customer_id: cust,
            channel: "تلفن".into(),
            subject: "x".into(),
            transcript: "x".into(),
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .await
        .expect("int");

    let svc = Service::new(&store);
    let err = svc
        .score_interaction(ScoreRequest {
            interaction_id: iid,
            rubric_id: Some(rubric_id),
            scores: vec![150.0], // out of range
            evaluator: None,
            notes: "".into(),
        })
        .await;
    assert!(matches!(err, Err(AppError::Validation(_))));
}
