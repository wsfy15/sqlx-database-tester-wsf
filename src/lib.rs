use std::{path::Path, thread};

use sqlx::{migrate::Migrator, Connection, Executor, PgConnection, PgPool};
use tokio::runtime::Runtime;
use uuid::Uuid;

pub struct TestDB {
    pub dbname: String,
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl TestDB {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
        password: impl Into<String>,
        migration_path: impl Into<String>,
    ) -> Self {
        let host = host.into();
        let username = username.into();
        let password = password.into();
        let migration_path = migration_path.into();

        // create random database
        let uuid = Uuid::new_v4();
        let dbname = format!("test-{}", uuid);
        let dbname_cloned = dbname.clone();
        let config = TestDB {
            dbname,
            host,
            port,
            username,
            password,
        };
        let server_url = config.server_url();
        let url = config.url();

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let mut conn = PgConnection::connect(&server_url).await.unwrap();

                // r# # 创建原始字符串字面量 不需要进行转义，如果字符串包含#字符， 可以使用 r#### #### 方式，只要保证首尾#数量相同
                conn.execute(format!(r#"CREATE DATABASE "{}""#, dbname_cloned).as_str())
                    .await
                    .unwrap();

                // execute migration on new database
                let mut conn = PgConnection::connect(&url).await.unwrap();

                let migrator = Migrator::new(Path::new(&migration_path)).await.unwrap();
                migrator.run(&mut conn).await.unwrap();
            })
        })
        .join()
        .expect("failed to create database");

        config
    }

    pub fn server_url(&self) -> String {
        if self.password.is_empty() {
            format!("postgres://{}@{}:{}", self.username, self.host, self.port)
        } else {
            format!(
                "postgres://{}:{}@{}:{}",
                self.username, self.password, self.host, self.port
            )
        }
    }

    pub fn url(&self) -> String {
        format!("{}/{}", self.server_url(), self.dbname)
    }

    pub async fn get_pool(&self) -> PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.url())
            .await
            .unwrap()
    }
}

impl Drop for TestDB {
    fn drop(&mut self) {
        let server_url = self.server_url();
        let db_name = self.dbname.clone();

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
        rt.block_on(async {
          let mut conn = PgConnection::connect(&server_url).await.unwrap();

          // terminate existing connections
          sqlx::query(&format!(r#"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND datname = '{}'"#, db_name))
          .execute(&mut conn)
          .await
          .expect("Terminate all other connections");

          conn.execute(format!(r#"DROP DATABASE "{}""#, db_name).as_str()).await.expect("Error while querying the drop database");
        })}).join().expect("fail to drop database");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db_should_create_and_drop() {
        // fixtures 文件路径相对于 Cargo.toml
        let tdb = TestDB::new("localhost", 5432, "postgres", "123456", "./migrations");
        let pool = tdb.get_pool().await;

        sqlx::query("INSERT INTO todos (title) VALUES ('test')")
            .execute(&pool)
            .await
            .unwrap();
        let (id, title) = sqlx::query_as::<_, (i32, String)>("SELECT id, title from todos")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(1, id);
        assert_eq!("test", title);
    }
}
