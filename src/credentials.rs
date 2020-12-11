use std::collections::HashMap;
use std::default::Default;

use async_std::task;
use dynomite::{Item, FromAttributes};
use dynomite::dynamodb::{DynamoDb, DynamoDbClient, GetItemInput, GetItemError};
use rusoto_core::Region;
use thiserror::Error;
// use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, GetItemInput};
// use rusoto_dynamodb::{DynamoDbClient};

use crate::settings::Settings;


#[derive(Debug, Error)]
pub enum CredentialsError
{
    #[error("No credentials for token {0}")]
    MissingCredentials(String),

    // #[error(transparent)]
    // BadCredentials(#[from] dynomite::AttributeError),
    #[error("malformed credentials")]
    MalformedCredentials { source: dynomite::AttributeError },

    #[error(transparent)]
    DynamoDbError(#[from] rusoto_core::RusotoError<GetItemError>),
}

#[derive(Item, Clone)]
pub struct Credentials {
    #[dynomite(partition_key)]
    token: String,
    name: String,
    adapter: String,
    creds: HashMap<String, String>,
}

pub struct Store {
    settings: Settings,
    dynamo_db: rusoto_dynamodb::DynamoDbClient,
    cache: HashMap<String, Credentials>,
}

impl Store {
    pub fn new(settings: Settings) -> Store {
        Store {
            settings,
            dynamo_db: DynamoDbClient::new(Region::UsEast2),
            cache: HashMap::with_capacity(100),
        }
    }

    pub fn fetch(&mut self, token: &String) -> Result<Credentials,CredentialsError> {
        match self.cache.get_mut(token) {
            Some(creds) => Ok(creds.clone()),
            None => {
                task::block_on(async {
                    let creds = self.get_item(token).await?;
                    self.cache.insert(token.clone(), creds.clone());
                    Ok(creds)
                })
            }
        }
    }

    async fn get_item(&self, token: &String) -> Result<Credentials,CredentialsError> {
        let req = GetItemInput {
            table_name: self.settings.credentials_table.clone().into(),
            key: CredentialsKey { token: token.clone() }.into(),
            ..Default::default()
        };
        self.dynamo_db.get_item(req).await?.item
            .map(|item| Credentials::from_attrs(item)
                .map_err(|source| CredentialsError::MalformedCredentials { source })
                )
            .ok_or(CredentialsError::MissingCredentials(token.clone()))?
    }
}
