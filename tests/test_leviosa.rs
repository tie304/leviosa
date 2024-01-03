use std::time::Duration;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use ctor::{ctor, dtor};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    migrate::Migrator, postgres::PgPoolOptions, prelude::FromRow, PgPool
};
use super_macros::leviosa;
use uuid::Uuid;

#[leviosa]
#[derive(Debug, FromRow, Clone)]
struct TestStruct {
    id: Option<i32>,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct JsonFieldData {
    key1: String,
    value1: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct JsonbFieldData {
    key2: String,
    value2: bool,
}

#[derive(sqlx::Type, Debug, Clone, PartialEq)]
#[sqlx(type_name = "mood", rename_all = "lowercase")]
enum Mood {
    Sad,
    Ok,
    Happy,
}

#[leviosa]
#[derive(Debug, FromRow, Clone, PartialEq)]
struct MoreAdvancedStruct {
    id: Option<i32>,
    name: String,
    email: String,
    verified: bool,
    bio: Option<String>,
    created: DateTime<Utc>,
    small_int_field: Option<i16>, // Corresponds to SMALLINT in PostgreSQL
    integer_field: Option<i32>,   // Corresponds to INT in PostgreSQL
    big_int_field: Option<i64>,   // Corresponds to BIGINT in PostgreSQL

    // Floating point types
    float_field: Option<f32>,  // Corresponds to FLOAT/REAL in PostgreSQL
    double_field: Option<f64>, // Corresponds to DOUBLE PRECISION in PostgreSQL
    //numeric_field: Option<f64>,    // TODO support NUMERIC
    char_field: Option<String>,
    bytea_field: Option<Vec<u8>>,
    date_field: Option<NaiveDate>,
    time_field: Option<NaiveTime>,
    timestamp_field: Option<NaiveDateTime>,
    //inet_field: Option<IpAddr>, TODO Support IpAddr
    uuid_field: Option<Uuid>,
    int_array_field: Option<Vec<i32>>,
    mood_field: Option<Mood>,
    json_field: Option<Value>,
    jsonb_field: Option<Value>,
}

static MIGRATOR: Migrator = sqlx::migrate!();

async fn setup_database() -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect("postgres://postgres:mysecretpassword@localhost:5432/postgres")
        .await?;

    match MIGRATOR.run(&pool).await {
        Ok(_) => println!("Migrations applied successfully."),
        Err(e) => println!("Error applying migrations: {}", e),
    }

    Ok(pool)
}

async fn teardown_database() -> Result<(), sqlx::Error> {
    let pool = setup_database().await.unwrap();
    sqlx::query!("drop table if exists test_struct")
        .execute(&pool)
        .await?;
    sqlx::query!("drop table if exists more_advanced_struct")
        .execute(&pool)
        .await?;

    sqlx::query!("DROP TABLE IF EXISTS _sqlx_migrations")
        .execute(&pool)
        .await?;

    sqlx::query!("DROP TYPE mood").execute(&pool).await?;
    Ok(())
}

#[ctor]
fn global_setup() {
    println!("Global test setup");
}

#[dtor]
fn global_teardown() {
    println!("Global test teardown");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        teardown_database().await.unwrap();
    });
}

#[tokio::test]
async fn test_leviosa_macro_basic_crud() {
    let db = setup_database().await.expect("Database setup failed");

    let create_entity = TestStruct::create(&db, String::from("bob"))
        .await
        .expect("Failed to create entity");

    let get_entity = TestStruct::get_by_id(&db, &create_entity.id)
        .await
        .expect("Failed to get by id");

    assert!(get_entity.is_some());

    let mut get_entity = get_entity.unwrap();

    get_entity
        .update_name(&db, &String::from("bob"))
        .await
        .expect("updating failed");

    assert_eq!(get_entity.name, String::from("bob"));

    get_entity.delete(&db).await.unwrap();

    let entity = TestStruct::get_by_id(&db, &get_entity.id)
        .await
        .expect("Operation failed fetching by id");

    assert!(entity.is_none());
}
#[tokio::test]
async fn test_update_all_types() {
    let db = setup_database().await.expect("Database setup failed");

    let date = Utc::now();
    let new_date = Utc::now().with_nanosecond(0);
    let generated_uuid = uuid::Uuid::new_v4();

    let json_data = serde_json::json!({
        "key1": "value1",
        "number": 123
    });

    let jsonb_data = serde_json::json!({
        "key2": "value2",
        "flag": true
    });

    let mut entity = MoreAdvancedStruct::create(
        &db,
        String::from("First"),
        String::from("Last"),
        false,
        date,
    )
    .await
    .expect("Failed to create entity");

    //entity.update_id(&db, &Some(2)).await.expect("Could not update id");
    entity
        .update_created(&db, &new_date.unwrap())
        .await
        .expect("Could not not update date");
    entity
        .update_bio(&db, &Some(format!("MY BIO")))
        .await
        .expect("Could not update id");
    entity
        .update_name(&db, &String::from("New name"))
        .await
        .expect("Could not update name");
    entity
        .update_email(&db, &String::from("new@gmail.com"))
        .await
        .expect("Could not update email");
    entity
        .update_verified(&db, &true)
        .await
        .expect("Could not update verified");

    // Update integer types
    entity
        .update_small_int_field(&db, &Some(32767))
        .await
        .expect("Could not update small_int_field");
    entity
        .update_integer_field(&db, &Some(2147483647))
        .await
        .expect("Could not update integer_field");
    entity
        .update_big_int_field(&db, &Some(9223372036854775807))
        .await
        .expect("Could not update big_int_field");

    // Update float types
    entity
        .update_float_field(&db, &Some(123.45))
        .await
        .expect("Could not update float_field");
    entity
        .update_double_field(&db, &Some(678.90))
        .await
        .expect("Could not update double_field");

    entity
        .update_char_field(&db, &Some(String::from("Char Field")))
        .await
        .expect("Could not update char_field");
    entity
        .update_bytea_field(&db, &Some(vec![1, 2, 3, 4, 5]))
        .await
        .expect("Could not update bytea_field");
    entity
        .update_date_field(&db, &Some(chrono::NaiveDate::from_ymd(2023, 3, 15)))
        .await
        .expect("Could not update date_field");
    entity
        .update_time_field(&db, &Some(chrono::NaiveTime::from_hms(12, 0, 0)))
        .await
        .expect("Could not update time_field");
    entity
        .update_timestamp_field(
            &db,
            &Some(chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd(2023, 3, 15),
                chrono::NaiveTime::from_hms(12, 0, 0),
            )),
        )
        .await
        .expect("Could not update timestamp_field");
    entity
        .update_uuid_field(&db, &Some(generated_uuid))
        .await
        .expect("Could not update uuid_field");
    entity
        .update_int_array_field(&db, &Some(vec![1, 2, 3, 4, 5]))
        .await
        .expect("Could not update int_array_field");
    entity
        .update_mood_field(&db, &Some(Mood::Happy))
        .await
        .expect("Could not update mood field");
    entity
        .update_json_field(&db, &Some(json_data.clone()))
        .await
        .expect("Could not update json_field");
    entity
        .update_jsonb_field(&db, &Some(jsonb_data.clone()))
        .await
        .expect("Could not update jsonb_field");

    let fetched_entity = MoreAdvancedStruct::get_by_id(&db, &entity.id)
        .await
        .expect("Could get retreve by id")
        .unwrap();

    //assert_eq!(fetched_entity.id.unwrap(), 2);
    assert_eq!(fetched_entity.created, new_date.unwrap());
    assert_eq!(fetched_entity.bio.unwrap(), format!("MY BIO"));

    assert_eq!(fetched_entity.small_int_field, Some(32767));
    assert_eq!(fetched_entity.integer_field, Some(2147483647));
    assert_eq!(fetched_entity.big_int_field, Some(9223372036854775807));

    assert!((fetched_entity.float_field.unwrap() - 123.45).abs() < f32::EPSILON); // Comparing float with a small epsilon
    assert!((fetched_entity.double_field.unwrap() - 678.90).abs() < f64::EPSILON); // Comparing double with a small epsilon

    assert_eq!(fetched_entity.name, String::from("New name"));
    assert_eq!(fetched_entity.email, String::from("new@gmail.com"));
    assert_eq!(fetched_entity.verified, true);

    assert_eq!(fetched_entity.char_field, Some(String::from("Char Field")));
    assert_eq!(fetched_entity.bytea_field, Some(vec![1, 2, 3, 4, 5]));
    assert_eq!(
        fetched_entity.date_field,
        Some(chrono::NaiveDate::from_ymd(2023, 3, 15))
    );
    assert_eq!(
        fetched_entity.time_field,
        Some(chrono::NaiveTime::from_hms(12, 0, 0))
    );
    assert_eq!(
        fetched_entity.timestamp_field,
        Some(chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(2023, 3, 15),
            chrono::NaiveTime::from_hms(12, 0, 0)
        ))
    );

    assert_eq!(fetched_entity.uuid_field, Some(generated_uuid));
    assert_eq!(fetched_entity.int_array_field, Some(vec![1, 2, 3, 4, 5]));

    assert_eq!(fetched_entity.mood_field, Some(Mood::Happy));
    assert_eq!(fetched_entity.json_field.as_ref().unwrap(), &json_data);
    assert_eq!(fetched_entity.jsonb_field.as_ref().unwrap(), &jsonb_data);
}

#[tokio::test]
async fn test_find_all() {
    let db = setup_database().await.expect("Database setup failed");
    let first_entry = MoreAdvancedStruct::create(
        &db,
        String::from("My Name"),
        String::from("tylerhanson9123@gmail.com"),
        false,
        Utc::now(),
    )
    .await
    .expect("Failed to create entity");

    let verified = MoreAdvancedStruct::create(
        &db,
        String::from("My Name 2"),
        String::from("tylerhanson9123@gmail.com"),
        true,
        Utc::now(),
    )
    .await
    .expect("Failed to create entity");

    MoreAdvancedStruct::create(
        &db,
        String::from("My Name 3"),
        String::from("tylerhanson912@gmail.com"),
        false,
        Utc::now(),
    )
    .await
    .expect("Failed to create entity");

    let limit_entities = MoreAdvancedStruct::find()
        .limit(1)
        .execute(&db)
        .await
        .expect("Failed to fetch all");

    assert_eq!(limit_entities.len(), 1);

    let where_entities = MoreAdvancedStruct::find()
        .r#where("email ='tylerhanson9123@gmail.com' AND name = 'My Name'")
        .order_by("verified ASC")
        .execute(&db)
        .await
        .expect("Failed where Clause");

    assert_eq!(where_entities[0], first_entry);

    let ordered_by_entities = MoreAdvancedStruct::find()
        .order_by("verified DESC")
        .execute(&db)
        .await
        .expect("Failed where Clause");

    assert_eq!(ordered_by_entities[0].id, verified.id);
}

#[tokio::test]
async fn test_create_many() {
    todo!()
}

#[tokio::test]
async fn test_update_many() {
    todo!()
}

#[tokio::test]
async fn test_delete_many() {
    todo!()
}