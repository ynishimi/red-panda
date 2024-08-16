use std::str;
use std::fs;
use anyhow::Error;
use anyhow::{Result, Context};
use dialoguer::{Input, Password};
use reqwest::Client;
use scraper::{Html, Selector};

// use security_framework::passwords::{self, get_generic_password};
// use clap::Parser;

// type MyResult<T> = Result<T, Box<dyn Error>>;

// const SERVICE: &str = "com.apple.network.eap.user.item.wlan.ssid.KUINS-Air";
// TODO: change path to ~/.config
const CONFIG_FILE_PATH: &str = "config.yml";

#[derive(Debug)]
pub struct Credential {
    account: String,
    password: String,
    login_token: String,
}

pub async fn get_credential() -> Result<Credential> {
    let account;
    let file_result = fs::read(CONFIG_FILE_PATH);
    match file_result {
        Ok(file_value) => {
            account = String::from_utf8(file_value)?;
            println!("Log in as: {}", account);
        }
        Err(_) => {
            account = get_account()?;
        }
    }
    let password = get_password()?;
    let login_token = get_login_token().await?;
    Ok(Credential {
        account: account,
        password: password,
        login_token: login_token,
    })
}

fn get_account() -> Result<String> {
    let account: String = Input::new()
        .with_prompt("Enter your ECS-ID")
        .interact_text()?;

    fs::write(CONFIG_FILE_PATH, &account)?;
    Ok(account)
}

fn get_password() -> Result<String> {
    let password: String = Password::new()
        .with_prompt("Enter your password")
        .interact()?;
    Ok(password)
}

// get login token
pub async fn get_login_token() -> Result<String> {
    // let url = "https://panda.ecs.kyoto-u.ac.jp/portal/login";
    let url = "https://panda.ecs.kyoto-u.ac.jp/cas/login";
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .await?;
    let body = response.text().await?;
    let body_parse = Html::parse_document(&body);
    // TODO: remove unwrap()
    let selector = Selector::parse("input[name='lt']").unwrap();
    let lt_parse = body_parse.select(&selector)
        .next()
        .and_then(|input| input.value().attr("value"))
        .context("Couldn't get login token");
    match lt_parse {
        Ok(lt) => {
            println!("{}", lt);
            Ok(lt.to_string())
        },
        Err(e) => Err(e),
    }
}

pub fn login(credential: Credential) -> Result<()> {
    println!("{:?}", credential);
    Ok(())
}