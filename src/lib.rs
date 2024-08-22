use std::str;
use std::fs;
// use anyhow::Error;
use anyhow::{Result, Context};
use dialoguer::{Input, Password, Select};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use open;
use std::time::Duration;
use indicatif::ProgressBar;

const CONFIG_FILE_PATH: &str = "config.yml";
const PASSWORD_FILE_PATH: &str = "password.yml";

const LOGIN_URL: &str = "https://panda.ecs.kyoto-u.ac.jp/cas/login?service=https%3A%2F%2Fpanda.ecs.kyoto-u.ac.jp%2Fsakai-login-tool%2Fcontainer";
// const LOGIN_URL: &str = "https://panda.ecs.kyoto-u.ac.jp/cas/login";
const BASE_URL: &str =  "https://panda.ecs.kyoto-u.ac.jp";

#[derive(Debug)]
pub struct Credential {
    account: String,
    password: String,
}
#[derive(Debug, Deserialize)]
pub struct FavoriteCourses {
    #[serde(rename = "favoriteSiteIds")]
    pub favorite_site_ids: Vec<String>,
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

#[derive(Debug, Deserialize)]
pub struct SiteContentCollection {
    content_collection: Vec<SiteContent>,
}
impl SiteContentCollection {
    pub fn get(&self) -> Option<&SiteContent> {
        self.content_collection.get(0)
    }
}

#[derive(Debug, Deserialize)]
pub struct SiteContent {
    name: String,
    #[serde(rename = "resourceChildren")]
    pub resource_children: Vec<ResourceChildren>,
    // #[serde(rename = "mimeType")]
    // mime_type: String,
    // modified: String,
    // url: String,
}
#[derive(Debug, Deserialize)]
pub struct ResourceChildren {
    name: String,
    #[serde(rename = "mimeType")]
    _mime_type: String,
    // modified: String,
    pub url: String,
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
    let password;
    let password_file_result = fs::read(PASSWORD_FILE_PATH);
    match password_file_result {
        Ok(file_value) => {
            password = String::from_utf8(file_value)?;
        }
        Err(_) => {
            password = get_password()?;
        }
    }
    // let password = get_password()?;
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

    // TODO: delete this feature
    fs::write(PASSWORD_FILE_PATH, &password)?;
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

pub async fn login(client: &Client, credential: &Credential) -> Result<String> {
    let login_token = get_login_token(&client).await?;
    let params = [
        ("username", credential.account.as_str()),
        ("password", credential.password.as_str()),
        ("lt", login_token.as_str()),
        ("execution", "e1s1"),
        ("_eventId", "submit"),
        ("submit", "ログイン"),
    ];
    
    let bar = ProgressBar::new_spinner().with_message("Logging in...");
    bar.enable_steady_tick(Duration::from_millis(100));

    let _res_login= client
        .post(LOGIN_URL)
        .form(&params)
        .send()
        .await?;
    // println!("{:?}", res_login);

    // // check if the login succeeded
    let res_session: SessionResponse = client
        // .get(BASE_URL.to_owned() + "/session.json")
        .get(format!("{}/direct/session.json", BASE_URL))
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

    bar.finish_and_clear();
    Ok(current_session)
    // Ok(current_session)
}

pub async fn get_favorite_courses(client: &Client) -> Result<FavoriteCourses> {
    let bar = ProgressBar::new_spinner().with_message("Getting courses...");
    bar.enable_steady_tick(Duration::from_millis(100));
    // https://panda.ecs.kyoto-u.ac.jp/portal/favorites/list
    let favorite_courses = client
        .get(format!("{}/portal/favorites/list", BASE_URL))
        .send()
        .await?
        .json::<FavoriteCourses>()
        .await?;
    bar.finish_and_clear();
    Ok(favorite_courses)
}

pub async fn get_site_content(client: &Client, site_id: &String) -> Result<SiteContentCollection> {
    let bar = ProgressBar::new_spinner().with_message("Getting courses...");
    bar.enable_steady_tick(Duration::from_millis(100));
    println!("{}",format!("{}/direct/content/resources/{}.json", BASE_URL, site_id));
    let site_content = client
        .get(format!("{}/direct/content/resources/{}.json", BASE_URL, site_id))
        .send()
        .await?
        .json::<SiteContentCollection>()
        .await?;
    bar.finish_and_clear();
    Ok(site_content)
}
pub fn select_site(favorite_courses: &FavoriteCourses) -> Result<usize> {
    let items = &favorite_courses.favorite_site_ids;
    let selection = Select::new()
    .with_prompt("Choose course")
    .default(0)
    .items(&items)
    .interact()?;
    println!("You chose: {}", items[selection]); 
    Ok(selection)
}

pub fn select_child_site(site_content_children: &Vec<ResourceChildren>) -> Result<usize> {
    let items: Vec<&str> = site_content_children.iter().map(|child| child.name.as_str()).collect();
    let selection = Select::new()
    .default(0)
    .items(&items)
    .interact()?;
    println!("You chose: {}", items[selection]); 
    Ok(selection)
}

pub fn open_in_browser(url: &String) -> Result<()> {
    open::that(url)?;
    Ok(())
}