use std::str;
use std::fs;
use anyhow::{Result, Context};
use dialoguer::{Input, Password};
use reqwest::Client;
use scraper::{Html, Selector};
// use serde::{Serialize, Deserialize};
use serde_urlencoded;
// use security_framework::passwords::{self, get_generic_password};
// use clap::Parser;

// type MyResult<T> = Result<T, Box<dyn Error>>;

// const SERVICE: &str = "com.apple.network.eap.user.item.wlan.ssid.KUINS-Air";
// TODO: change path to ~/.config
const CONFIG_FILE_PATH: &str = "config.yml";

const LOGIN_URL: &str = "https://panda.ecs.kyoto-u.ac.jp/cas/login?service=https%3A%2F%2Fpanda.ecs.kyoto-u.ac.jp%2Fsakai-login-tool%2Fcontainer";
// const BASE_URL: &str =  "https://panda.ecs.kyoto-u.ac.jp/direct";
#[derive(Debug)]
pub struct Credential {
    account: String,
    password: String,
    // login_token: String,
}

// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// struct RequestBody {
//     username: String,
//     password: String,
//     lt: String,
//     execution: String,
//     _eventId: String,
//     submit: String,
// }

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
    // let url = "https://panda.ecs.kyoto-u.ac.jp/portal/login";

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
            // println!("{}", lt);
            Ok(lt.to_string())
        },
        Err(e) => Err(e),
    }
}

pub async fn login(client: &Client, credential: Credential) -> Result<()> {
    println!("{:?}", credential);
    let login_token = get_login_token(&client).await?;
    // let request_body = RequestBody {
    //     username: credential.account,
    //     password: credential.password,
    //     lt: login_token,
    //     execution: String::from("e1s1"),
    //     _eventId: String::from("submit"),
    //     submit: String::from("ログイン"),
    // };
    let params = [
        ("username", credential.account.as_str()),
        ("password", credential.password.as_str()),
        ("lt", login_token.as_str()),
        ("execution", "e1s1"),
        ("_eventId", "submit"),
        ("submit", "ログイン"),
    ];
    // let request_encoded = serde_urlencoded::to_string(&request_body)?;
    // println!("{:?}", request_encoded);
    let res_login = client
        .post(LOGIN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await?
        .text()
        .await?;

    // check if the login succeeded
    let res = client
        .get("https://panda.ecs.kyoto-u.ac.jp/direct/session.json")
        .send()
        .await?
        .text()
        .await?;
    println!("{}", res);
    
    Ok(())
}