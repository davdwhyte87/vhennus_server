use chrono::Local;
use log::info;
use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct AppScheduler {
    pub inner:JobScheduler
}

impl AppScheduler {
    pub async fn build(pool:PgPool)-> anyhow::Result<Self> {
        let sched = JobScheduler::new().await?;
        let p1 = pool.clone();
        let job1 = Job::new_async_tz(
            "0 45 1 * * * *",
            Local,
            move |_uuid, _l| {
            let p1 = p1.clone();
            Box::pin(async move {
                info!("[Job 1] daily at 22:00: {}", chrono::Local::now());
                // your daily notify…
                Self::morning_jobs(p1).await
            })
        })?;
        sched.add(job1).await?;

        Ok(Self { inner: sched })
    }

    /// Start the background scheduler (non‐blocking)
    pub async fn start(self) {
        // this fires jobs on the shared Tokio runtime
        let handle = self.inner.clone();
        tokio::spawn(async move {
            handle.start().await.unwrap();
        });
    }
    
    pub async fn morning_jobs(pool:PgPool){
        log::info!("[Job 1] morning_jobs running ......");
    }
}