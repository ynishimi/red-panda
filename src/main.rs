use red_panda::{get_credential, login};
use tokio;
use anyhow::{Result, Context};
use reqwest::{Client, RequestBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    // let client = Client::new();
    let client = Client::builder().cookie_store(true).build()?;
    let credential = get_credential()?;
    login(&client, credential).await?;
    Ok(())
}
