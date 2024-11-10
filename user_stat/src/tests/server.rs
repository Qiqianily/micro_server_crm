use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use chrono::{TimeZone, Utc};
use futures::StreamExt;
use sqlx_db_tester::TestPg;
use tokio::time::sleep;
use tonic::transport::Server;
use user_stat::{
    pb::{user_stats_client::UserStatsClient, QueryRequestBuilder, RawQueryRequestBuilder},
    test_utils::{id, tq},
    UserStatsService,
};

const PORT_BASE: u32 = 60000;

/// 这个测试 会随着时间推移产生问题
#[tokio::test]
async fn query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server(PORT_BASE).await?;
    let mut client = UserStatsClient::connect(format!("http://{}", addr)).await?;

    // 减去额定时间 保证用例可用
    let created_date = Utc.with_ymd_and_hms(2023, 8, 1, 0, 0, 0).unwrap();
    let days = Utc::now().signed_duration_since(created_date).num_days();

    let query = QueryRequestBuilder::default()
        .timestamp(("created_at".to_string(), tq(Some(days), None)))
        .id(("viewed_but_not_started".to_string(), id(&[252790])))
        .build()
        .unwrap();

    let stream = client.query(query).await?.into_inner();
    let ret = stream.collect::<Vec<_>>().await;
    assert_eq!(ret.len(), 16);
    Ok(())
}

#[tokio::test]
async fn raw_query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server(PORT_BASE + 1).await?;
    let mut client = UserStatsClient::connect(format!("http://{}", addr)).await?;
    let raw_query = RawQueryRequestBuilder::default()
        .query("SELECT * FROM user_stats WHERE created_at > '2024-01-01' LIMIT 5")
        .build()?;

    let stream = client.raw_query(raw_query).await?.into_inner();
    let ret = stream
        .then(|res| async move { res.unwrap() })
        .collect::<Vec<_>>()
        .await;
    assert_eq!(ret.len(), 5);
    Ok(())
}

async fn start_server(port: u32) -> Result<(TestPg, SocketAddr)> {
    let addr = format!("[::1]:{}", port).parse()?;
    let (tdb, svc) = UserStatsService::new_for_test().await?;

    tokio::spawn(async move {
        Server::builder()
            .add_service(svc.into_server())
            .serve(addr)
            .await
            .unwrap();
    });

    sleep(Duration::from_micros(1)).await;

    Ok((tdb, addr))
}
