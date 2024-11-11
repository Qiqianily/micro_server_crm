mod abi;
mod config;
pub mod pb;

pub use config::AppConfig;

use futures::Stream;
use pb::{
    user_stats_server::{UserStats, UserStatsServer},
    QueryRequest, RawQueryRequest, User,
};
use sqlx::PgPool;
use std::{ops::Deref, pin::Pin, sync::Arc};
use tonic::{async_trait, Request, Response, Status};

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;

/// UserStatsService
#[derive(Clone)]
pub struct UserStatsService {
    inner: Arc<UserStatsServiceInner>,
}

/// 内部数据，此数据通过Arc方式Clone
#[allow(dead_code)]
pub struct UserStatsServiceInner {
    config: AppConfig,
    pool: PgPool,
}

/// 为UserStats实现UserStats RPC Trait
#[async_trait]
impl UserStats for UserStatsService {
    // 实现QueryStream
    type QueryStream = ResponseStream;
    // Query
    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        let query = request.into_inner();
        self.query(query).await
    }

    // 实现RawQueryStream
    type RawQueryStream = ResponseStream;
    // RawQuert
    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let query = request.into_inner();
        self.raw_query(query).await
    }
}

impl UserStatsService {
    // 实现创建一个新的Service实例
    pub async fn new(config: AppConfig) -> Self {
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .expect("Failed Connect to DB");
        let inner = UserStatsServiceInner { config, pool };

        Self {
            inner: Arc::new(inner),
        }
    }

    // 将 Service 转换为 RPC Server
    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::pb::{IdQuery, TimeQuery};
    use chrono::Utc;
    use prost_types::Timestamp;

    // impl UserStatsService {
    //     pub async fn new_for_test() -> Result<(TestPg, Self)> {
    //         let config = AppConfig::load()?;
    //         let post = config.server.db_url.rfind("/").expect("invalid db_url");
    //         let server_url = &config.server.db_url[..post];
    //         let (tdb, pool) = get_test_pool(Some(server_url)).await;
    //         let svc = Self {
    //             inner: Arc::new(UserStatsServiceInner { config, pool }),
    //         };
    //
    //         Ok((tdb, svc))
    //     }
    // }
    //
    // pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
    //     let url = match url {
    //         Some(url) => url.to_string(),
    //         None => "postgres://:123456@localhost:5432".to_string(),
    //     };
    //
    //     let p = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("migrations");
    //     let tdb = TestPg::new(url, p);
    //     let pool = tdb.get_pool().await;
    //
    //     let sql = include_str!("../fixtures/data.sql").split(";");
    //     let mut ts = pool.begin().await.expect("begin transaction failed");
    //     for s in sql {
    //         if s.trim().is_empty() {
    //             continue;
    //         } else {
    //             ts.execute(s).await.expect("execute sql failed");
    //         }
    //     }
    //
    //     ts.commit().await.expect("commit transaction failed");
    //
    //     (tdb, pool)
    // }

    pub fn id(id: &[u32]) -> IdQuery {
        IdQuery { ids: id.to_vec() }
    }
    pub fn tq(lower: Option<i64>, upper: Option<i64>) -> TimeQuery {
        TimeQuery {
            lower: lower.map(to_ts),
            upper: upper.map(to_ts),
        }
    }

    pub fn to_ts(days: i64) -> Timestamp {
        let dt = Utc::now()
            .checked_sub_signed(chrono::Duration::days(days))
            .unwrap();
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
}
