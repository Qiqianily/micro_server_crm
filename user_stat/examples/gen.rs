use anyhow::Result;
use chrono::{DateTime, Days, Utc};
use fake::faker::chrono::en::DateTimeBetween;
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::zh_cn::Name;
use fake::{Dummy, Fake, Faker};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

// generate 1000 fake users and run then in a tx,repeat 500 times
/**
create table user_stats (
    email varchar(128) not null primary key,
    name varchar(64) not null,
    created_at timestamptz not null default current_timestamp,
    last_visited_at timestamptz not null,
    last_watched_at timestamptz not null,
    recent_watched int[],
    viewed_but_not_started int[],
    started_but_not_finished int[],
    finished int[],
    last_email_notification timestamptz not null,
    last_in_app_notification timestamptz not null,
    last_sms_notification timestamptz not null
);
*/
#[derive(Debug, Clone, Dummy, Serialize, Deserialize, PartialEq, Eq)]
enum Gender {
    Female,
    Male,
    Unknown,
}

#[derive(Debug, Clone, Dummy, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserStat {
    #[dummy(faker = "UniqueEmail")]
    email: String,
    #[dummy(faker = "Name()")]
    name: String,
    gender: Gender,
    // #[dummy(faker = "DateTimeBetween(start_date='-30d', end_date='now')")]
    #[dummy(faker = "DateTimeBetween(before(365*5), now())")]
    created_at: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(30), now())")]
    last_visited_at: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(90), now())")]
    last_watched_at: DateTime<Utc>,
    #[dummy(faker = "IntList(50, 100000, 100000)")]
    recent_watched: Vec<i32>,
    #[dummy(faker = "IntList(50, 200000, 100000)")]
    viewed_but_not_started: Vec<i32>,
    #[dummy(faker = "IntList(50, 300000, 100000)")]
    started_but_not_finished: Vec<i32>,
    #[dummy(faker = "IntList(50, 400000, 100000)")]
    finished: Vec<i32>,
    #[dummy(faker = "DateTimeBetween(before(45), now())")]
    last_email_notification: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(15), now())")]
    last_in_app_notification: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(90), now())")]
    last_sms_notification: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // let user: UserStat = Faker.fake();
    // println!("{:?}", user);
    let pool = PgPool::connect("postgres://:123456@localhost:5432/stats").await?;
    for i in 1..=20 {
        let users: HashSet<UserStat> = (0..10000).map(|_| Faker.fake::<UserStat>()).collect();
        let start_time = std::time::Instant::now();
        // println!("{}...{:?}", i, users);
        raw_insert(users, &pool).await?;
        println!("Batch {} inserted in {:?}", i, start_time.elapsed());
    }
    Ok(())
}

async fn raw_insert(users: HashSet<UserStat>, pool: &PgPool) -> Result<()> {
    let mut sql = String::with_capacity(10 * 1000 * 1000);
    sql.push_str("
    INSERT INTO user_stats(email, name, gender, created_at, last_visited_at, last_watched_at, recent_watched, viewed_but_not_started, started_but_not_finished, finished, last_email_notification, last_in_app_notification, last_sms_notification)
    VALUES");
    for user in users {
        let gender = match user.gender {
            Gender::Female => "female",
            Gender::Male => "male",
            Gender::Unknown => "unknown",
        };
        sql.push_str(&format!(
            "('{}', '{}', '{}','{}', '{}', '{}', {}::int[], {}::int[], {}::int[], {}::int[], '{}', '{}', '{}'),",
            user.email,
            user.name,
            gender,
            user.created_at,
            user.last_visited_at,
            user.last_watched_at,
            list_to_string(user.recent_watched),
            list_to_string(user.viewed_but_not_started),
            list_to_string(user.started_but_not_finished),
            list_to_string(user.finished),
            user.last_email_notification,
            user.last_in_app_notification,
            user.last_sms_notification,
        ));
    }

    let v = &sql[..sql.len() - 1];
    sqlx::query(v).execute(pool).await?;

    Ok(())
}

fn list_to_string(list: Vec<i32>) -> String {
    format!("ARRAY{:?}", list)
}

#[allow(dead_code)]
async fn bulk_insert(users: HashSet<UserStat>, pool: &PgPool) -> Result<()> {
    // insert users into db
    let mut tx = pool.begin().await?;
    for user in users {
        let query = sqlx::query(
            r#"insert into user_stats (email, name,created_at, last_visited_at, last_watched_at, recent_watched, viewed_but_not_started, started_but_not_finished,finished, last_email_notification, last_in_app_notification, last_sms_notification) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
        )
       .bind(user.email)
       .bind(user.name)
       // .bind(user.gender)
       .bind(user.created_at)
       .bind(user.last_visited_at)
       .bind(user.last_watched_at)
       .bind(user.recent_watched)
       .bind(user.viewed_but_not_started)
       .bind(user.started_but_not_finished)
       .bind(user.finished)
       .bind(user.last_email_notification)
       .bind(user.last_in_app_notification)
       .bind(user.last_sms_notification);
        tx.execute(query).await?;
    }
    tx.commit().await?;
    Ok(())
}

// 实现 PartialEq 保证 email 唯一性
impl Hash for UserStat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.email.hash(state);
    }
}

// 开始时间
fn before(days: u64) -> DateTime<Utc> {
    Utc::now().checked_sub_days(Days::new(days)).unwrap()
}
// 结束时间
fn now() -> DateTime<Utc> {
    Utc::now()
}

// 随机生成一个长度为size的整数列表，每个整数范围为[start, start+len)
struct IntList(pub i32, pub i32, pub i32);

impl Dummy<IntList> for Vec<i32> {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(v: &IntList, rng: &mut R) -> Vec<i32> {
        let (max, start, len) = (v.0, v.1, v.2);
        let size = rng.gen_range(0..max);
        (0..size)
            .map(|_| rng.gen_range(start..start + len))
            .collect()
    }
}

// 处理 email 字段，保证唯一性
struct UniqueEmail;
const ALPHABET: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

impl Dummy<UniqueEmail> for String {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &UniqueEmail, rng: &mut R) -> String {
        let email: String = SafeEmail().fake_with_rng(rng);
        let nanoid = nanoid!(8, &ALPHABET);
        let at = email.find('@').unwrap();
        format!("{}.{}@{}", &email[..at], nanoid, &email[at + 1..])
    }
}
