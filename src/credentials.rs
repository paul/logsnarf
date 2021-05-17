use std::default::Default;

// use dynomite::dynamodb::{DynamoDb, GetItemError, GetItemInput};
// use dynomite::{Attributes, FromAttributes, Item};
use dynomite::{
    dynamodb::{DynamoDb, GetItemError, GetItemInput},
    Attributes, Item, FromAttributes
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("No credentials for token {0}")]
    MissingCredentials(String),

    // #[error(transparent)]
    // BadCredentials(#[from] dynomite::AttributeError),
    #[error("malformed credentials")]
    MalformedCredentials { source: dynomite::AttributeError },

    #[error(transparent)]
    DynamoDbError(#[from] rusoto_core::RusotoError<GetItemError>),
}

#[derive(Item, Clone, Debug)]
pub struct Credentials {
    #[dynomite(partition_key)]
    pub token: String,
    pub name: String,

    pub credentials: Secrets,
}

#[derive(Attributes, Clone, Debug)]
#[dynomite(tag = "adapter")]
pub enum Secrets {
    #[dynomite(rename = "influxdb_v1")]
    InfluxDbCredentials(InfluxDbCredentials)
}


#[derive(Attributes, Clone, Debug)]
pub struct InfluxDbCredentials {
    #[dynomite(rename = "type")]
    pub adapter: String,
    pub influxdb_url: String,
}

pub async fn fetch(client: &dyn DynamoDb, table_name: String, token: &String) -> Result<Credentials, CredentialsError> {
    let req = GetItemInput {
        table_name,
        key: CredentialsKey { token: token.clone() }.into(),
        ..Default::default()
    };
    client.get_item(req).await?.item
        .map(|item| {
            Credentials::from_attrs(&mut item.clone())
                .map_err(|source| CredentialsError::MalformedCredentials { source })
        })
        .ok_or(CredentialsError::MissingCredentials(token.clone()))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::assert_ok;

    use dynomite::dynamodb::DynamoDbClient;
    use rusoto_core::Region;

    #[tokio::test]
    async fn test_fetch() {
        let client = DynamoDbClient::new(Region::UsEast2);
        let table = "logsnarf_config".to_string();
        let token = "e0ff2e6751893dcd7fcb7a94d4535437".to_string();
        let result = fetch(&client, table, &token).await;
        println!("{:#?}", result);
        assert_ok!(result);
        let creds = result.unwrap();
        assert_eq!(creds.token, token);
        match creds.credentials {
            Secrets::InfluxDbCredentials(secrets) => {
                assert_eq!(secrets.adapter, "influxdb_v1".to_string());
                assert_eq!(secrets.influxdb_url, "http://localhost:8086/logsnarf".to_string());
            },
            _ => assert!(false, "secrets wasn't an InfluxDbCredentials")

        }
    }
}
