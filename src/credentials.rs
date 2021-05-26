use log::debug;
use sqlx::types::Json;
use serde::{Serialize, Deserialize};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("No credentials for token {0}")]
    MissingCredentials(String),

    #[error(transparent)]
    BadCredentials(#[from] sqlx::Error),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Secrets {
    InfluxDbV1 { url: String }
}

#[derive(Clone, Debug)]
pub struct Credentials {
    pub token: String,
    pub name: String,

    pub r#type: String,

    pub secrets: Secrets,
}

#[derive(sqlx::FromRow, Clone, Debug)]
struct CredentialsRow {
    pub token: String,
    pub name: String,
    pub r#type: String,

    pub secrets: sqlx::types::Json<Secrets>,
}

pub async fn fetch(token: &String, mut conn: sqlx::PgConnection) -> Result<Credentials, CredentialsError> {

    let row = sqlx::query_as!(
        CredentialsRow,
        r#"SELECT token, name, type, secrets AS "secrets: Json<Secrets>" FROM credentials WHERE token = $1 LIMIT 1"#,
        token
        )
        .fetch_optional(&mut conn)
        .await?;

    debug!("{:?}", row);

    row
        .ok_or(CredentialsError::MissingCredentials(token.to_owned()))
        .map(|CredentialsRow { token, name, r#type, secrets: Json(secrets) }| Credentials { token, name, r#type, secrets })
}

