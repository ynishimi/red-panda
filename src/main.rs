use red_panda::{get_credential, login};
use tokio;
use anyhow::{Result, Context};
use reqwest::{Client, RequestBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    // let client = Client::new();
    let client = Client::builder().cookie_store(true).build()?;
    let credential = get_credential()?;
    match login(&client, &credential).await {
    // if let Ok(user) = login(&client, &credential).await {
        Ok(user) => println!("User: {}", user),
        Err(e) => eprintln!("{}", e),
    }
    Ok(())
}
