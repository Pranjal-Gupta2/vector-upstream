#[macro_use]
extern crate log;

use azure_identity::{DefaultAzureCredential, TokenCredential};
use azure_storage::core::prelude::*;
use azure_storage_blobs::prelude::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();
    // First we retrieve the account name, container and blob name from command line args

    let account = std::env::args()
        .nth(1)
        .expect("please specify the account name as first command line parameter");
    let container = std::env::args()
        .nth(2)
        .expect("please specify the container name as second command line parameter");
    let blob = std::env::args()
        .nth(3)
        .expect("please specify the blob name as third command line parameter");

    let bearer_token = DefaultAzureCredential::default()
        .get_token("https://storage.azure.com/")
        .await?;

    let http_client = azure_core::new_http_client();
    let blob_client = StorageAccountClient::new_bearer_token(
        http_client.clone(),
        &account,
        bearer_token.token.secret(),
    )
    .as_container_client(&container)
    .as_blob_client(&blob);

    trace!("Requesting blob");

    let response = blob_client.get().execute().await?;

    let s_content = String::from_utf8(response.data.to_vec())?;
    println!("blob == {:?}", blob);
    println!("s_content == {}", s_content);

    Ok(())
}