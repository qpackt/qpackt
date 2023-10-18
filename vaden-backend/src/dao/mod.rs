use crate::error::{Result, VadenError};
use sqlx::Connection;
use sqlx::SqliteConnection;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Default file name with main vaden's database.
const SQLITE_FILE: &str = "vaden.sqlite";

/// DAO to encapsulate reading/writing from/to a database.
#[derive(Clone)]
pub(crate) struct Dao {
    inner: Arc<DaoInner>,
}

struct DaoInner {
    url: String,
}

impl Dao {
    pub(crate) async fn init(base_dir: &Path) -> Result<Self> {
        ensure_app_dir_exists(base_dir)?;
        let sqlite = base_dir.join(SQLITE_FILE);
        let path = sqlite
            .to_str()
            .ok_or_else(|| VadenError::DatabaseError("Non-UTF-8 file system detected".into()))?;
        let url = format!("sqlite://{path}?mode=rwc");
        let dao = Self {
            inner: Arc::new(DaoInner { url }),
        };
        dao.ensure_sqlite_initialized().await?;
        Ok(dao)
    }

    /// Registers new version of the site in database.
    ///
    /// Args:
    /// * web_root - full path to where the files are stored
    /// * name - name of the version
    pub(crate) async fn register_version(&self, web_root: &str, name: &str) -> Result<()> {
        let q = sqlx::query("INSERT INTO versions (web_root, name) values ($1, $2)")
            .bind(web_root)
            .bind(name);
        let mut connection = self.get_sqlite_connection().await?;
        q.execute(&mut connection)
            .await
            .map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_sqlite_connection(&self) -> Result<SqliteConnection> {
        SqliteConnection::connect(&self.inner.url)
            .await
            .map_err(|e| VadenError::DatabaseError(e.to_string()))
    }

    /// Called on startup to ensure that sqlite file exists and all migrations are applied.
    async fn ensure_sqlite_initialized(&self) -> Result<()> {
        let mut conn = self.get_sqlite_connection().await?;
        sqlx::migrate!("db/migrations")
            .run(&mut conn)
            .await
            .map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

// TODO move to more general module, this has nothing to do with dao
fn ensure_app_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        create_app_dir(path)
    } else if !path.is_dir() {
        Err(VadenError::InvalidConfig(format!(
            "App dir is not a directory: {path:?}"
        )))
    } else {
        Ok(())
    }
}

fn create_app_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|e| {
        VadenError::InvalidConfig(format!(
            "Unable to create app directory {}: {}",
            path.to_string_lossy(),
            e
        ))
    })
}
