use std::sync::Arc;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    let sem = Arc::new(Semaphore::new(10));
    sem.add_permits(5);
    
    let sem2 = sem.clone();
    tokio::spawn(async move {
        if let Ok(permit) = sem2.acquire_many(3).await {
            permit.forget();
        }
    });
}
