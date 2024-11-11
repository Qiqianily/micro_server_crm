use crate::{
    pb::{QueryRequest, QueryRequestBuilder, RawQueryRequest, TimeQuery, User},
    ResponseStream, ServiceResult, UserStatsService,
};
use chrono::{DateTime, TimeZone, Utc};
use core::fmt;
use itertools::Itertools;
use prost_types::Timestamp;
use std::collections::HashMap;
use tonic::{Response, Status};
use tracing::info;

// 实现UserStatsService内部函数
impl UserStatsService {
    // 条件查询
    pub async fn query(&self, query: QueryRequest) -> ServiceResult<ResponseStream> {
        let sql = query.to_string();
        // 调用raw_query
        self.raw_query(RawQueryRequest { query: sql }).await
    }

    // 原始SQL查询
    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        // SQLX 拿到列表
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.inner.pool)
            .await
        else {
            return Err(Status::internal(format!(
                "Failed to fetch data with query:{}",
                req.query
            )));
        };

        // 将 Iterator 转换为 Stream
        Ok(Response::new(Box::pin(futures::stream::iter(
            ret.into_iter().map(Ok),
        ))))
    }
}

/// 实现 Display 将 QueryRequest 转换为SQL
impl fmt::Display for QueryRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sql = "SELECT email, name FROM user_stats WHERE ".to_string();

        let time_conditions = self
            .time_stamps
            .iter()
            .map(|(k, v)| timestamp_query(k, v.lower.as_ref(), v.upper.as_ref()))
            .join(" AND ");

        sql.push_str(&time_conditions);

        let id_conditions = self
            .ids
            .iter()
            .map(|(k, v)| ids_query(k, &v.ids))
            .join(" AND ");

        if !id_conditions.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&id_conditions);
        }

        info!("Generated SQL: {}", sql);

        write!(f, "{}", sql)
    }
}

impl QueryRequest {
    pub fn new_with_dt(name: &str, lower: DateTime<Utc>, upper: DateTime<Utc>) -> Self {
        let ts = Timestamp {
            seconds: lower.timestamp(),
            nanos: 0,
        };
        let ts1 = Timestamp {
            seconds: upper.timestamp(),
            nanos: 0,
        };

        let tq = TimeQuery {
            lower: Some(ts),
            upper: Some(ts1),
        };
        // 创建一个 HashMap
        let mut time_stamps_map = HashMap::new();
        // 将 (name.to_string(), tq) 插入 HashMap
        time_stamps_map.insert(name.to_string(), tq);

        QueryRequestBuilder::default()
            .time_stamps(time_stamps_map)
            .build()
            .expect("Failed to build query request")
    }
}

// 组ID条件
fn ids_query(name: &str, ids: &Vec<u32>) -> String {
    if ids.is_empty() {
        return "TRUE".to_string();
    }

    format!("array{:?} <@ {}", ids, name)
}

// 组时间戳条件
fn timestamp_query(name: &str, lower: Option<&Timestamp>, upper: Option<&Timestamp>) -> String {
    if lower.is_none() && upper.is_none() {
        return "TRUE".to_string();
    }

    if lower.is_none() {
        let upper = ts_to_utc(upper.unwrap());
        return format!("{} <= '{}'", name, upper.to_rfc3339());
    }

    if upper.is_none() {
        let lower = ts_to_utc(lower.unwrap());
        return format!("{} >= '{}'", name, lower.to_rfc3339());
    }

    format!(
        "{} BETWEEN '{}' AND '{}'",
        name,
        ts_to_utc(lower.unwrap()).to_rfc3339(),
        ts_to_utc(upper.unwrap()).to_rfc3339()
    )
}

// 将时间戳 转换为UTC时间
fn ts_to_utc(ts: &Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as _).unwrap()
}

#[cfg(test)]

mod tests {
    use anyhow::Result;
    use futures::StreamExt;

    use super::*;
    use crate::{
        pb::QueryRequestBuilder,
        test_utils::{id, tq},
        AppConfig,
    };
    #[test]
    fn query_request_to_string_should_work() {
        let d1 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let d2 = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        let query = QueryRequest::new_with_dt("created_at", d1, d2);
        let sql = query.to_string();
        assert_eq!(
            sql,
            "SELECT email, name FROM user_stats WHERE created_at BETWEEN '2024-01-01T00:00:00+00:00' AND '2024-01-02T00:00:00+00:00'"
        );
    }

    #[tokio::test]
    async fn raw_query_should_work() -> Result<()> {
        let config = AppConfig::load().expect("Failed Load config");
        let svc = UserStatsService::new(config).await;
        let mut stream = svc
            .raw_query(RawQueryRequest {
                query: "select * from user_stats where created_at > '2024-01-01' limit 5"
                    .to_string(),
            })
            .await?
            .into_inner();

        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }

        Ok(())
    }

    #[tokio::test]
    async fn query_should_work() -> Result<()> {
        let config = AppConfig::load().expect("Failed Load config");
        let svc = UserStatsService::new(config).await;

        // 创建 HashMap 存储时间戳
        let mut time_stamps = HashMap::new();
        time_stamps.insert("created_at".to_string(), tq(Some(120), None));
        time_stamps.insert("last_visited_at".to_string(), tq(Some(30), None));

        let query = QueryRequestBuilder::default()
            .time_stamps(time_stamps)
            .id(("viewed_but_not_started".to_string(), id(&[252790])))
            .build()
            .expect("Failed to build query request");
        println!("generated query: {:?}", query);
        let mut stream = svc.query(query).await?.into_inner();
        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }
        Ok(())
    }
}
