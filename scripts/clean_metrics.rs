// scripts/clean_metrics.rs - delete metric/rubric/score/issue keys without touching agents/customers
use redis::AsyncCommands;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".into());
    let client = redis::Client::open(url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;

    // Find all keys, filter
    let all: Vec<String> = conn.keys("*").await?;
    let prefixes = ["mt:", "rb:", "sc:", "is:", "idx:metric", "set:metrics", "set:rubrics", "set:scores", "set:issues"];
    let to_del: Vec<&String> = all.iter().filter(|k| prefixes.iter().any(|p| k.starts_with(p))).collect();
    println!("Deleting {} keys:", to_del.len());
    for k in &to_del {
        println!("  {}", k);
    }
    if !to_del.is_empty() {
        let deleted: i64 = conn.del(to_del.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice()).await?;
        println!("Deleted {} keys from Redis", deleted);
    } else {
        println!("No matching keys to delete.");
    }
    Ok(())
}
