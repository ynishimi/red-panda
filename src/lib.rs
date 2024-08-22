use std::str;
use std::fs;
use anyhow::Error;
use anyhow::{Result, Context};
use dialoguer::{Input, Password};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;

const CONFIG_FILE_PATH: &str = "config.yml";

const LOGIN_URL: &str = "https://panda.ecs.kyoto-u.ac.jp/cas/login?service=https%3A%2F%2Fpanda.ecs.kyoto-u.ac.jp%2Fsakai-login-tool%2Fcontainer";
// const LOGIN_URL: &str = "https://panda.ecs.kyoto-u.ac.jp/cas/login";
const BASE_URL: &str =  "https://panda.ecs.kyoto-u.ac.jp/direct";

#[derive(Debug)]
pub struct Credential {
    account: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct SessionResponse {
    session_collection: Vec<Session>,
}
#[derive(Debug, Deserialize)]
struct Session {
    #[serde(rename = "userEid")]
    user_eid: Option<String>,
}


pub fn get_credential() -> Result<Credential> {
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
    Ok(Credential {
        account: account,
        password: password,
        // login_token: login_token,
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
pub async fn get_login_token(client: &Client) -> Result<String> {


    // let client = Client::new();
    let response = client
        .get(LOGIN_URL)
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
            Ok(lt.to_string())
        },
        Err(e) => Err(e),
    }
}

pub async fn login(client: &Client, credential: &Credential) -> Result<(String)> {
    let login_token = get_login_token(&client).await?;
    let params = [
        ("username", credential.account.as_str()),
        ("password", credential.password.as_str()),
        ("lt", login_token.as_str()),
        ("execution", "e1s1"),
        ("_eventId", "submit"),
        ("submit", "ログイン"),
    ];
    let res_login = client
        .post(LOGIN_URL)
        .form(&params)
        .send()
        .await?;
    // println!("{:?}", res_login);

    // // check if the login succeeded
    let res_session: SessionResponse = client
        // .get(BASE_URL.to_owned() + "/session.json")
        .get(format!("{}/session.json", BASE_URL))
        .send()
        .await?
        .json()
        .await?;
    let current_session = res_session.session_collection
        .get(0)
        .context("No sessions acquired")?
        .user_eid
        .clone()
        .context("Login failed")?;
    Ok((current_session))
    // Ok(current_session)
}