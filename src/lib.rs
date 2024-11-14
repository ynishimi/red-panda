use std::str;
use std::fs;
// use anyhow::Error;
use anyhow::{Result, Context};
use dialoguer::{Input, Password, Select, FuzzySelect};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use open;
// use serde_json::map;
use std::time::Duration;
use indicatif::ProgressBar;

const CONFIG_FILE_PATH: &str = "config.yml";
const PASSWORD_FILE_PATH: &str = "password.yml";

const LOGIN_URL: &str = "https://panda.ecs.kyoto-u.ac.jp/cas/login?service=https%3A%2F%2Fpanda.ecs.kyoto-u.ac.jp%2Fsakai-login-tool%2Fcontainer";
// const LOGIN_URL: &str = "https://panda.ecs.kyoto-u.ac.jp/cas/login";
const BASE_URL: &str =  "https://panda.ecs.kyoto-u.ac.jp";
const SEMINAR_RESOURCE_ID: &str = "/group/ae7eb08f-5eab-41d2-a8d8-229aac826b97/2024年度_定例セミナー _Weekly Seminar 2024_/";


#[derive(Debug)]
pub struct Credential {
    account: String,
    password: String,
}
#[derive(Debug, Deserialize)]
struct FavoriteSiteIds {
    #[serde(rename = "favoriteSiteIds")]
    pub favorite_site_ids: Vec<String>,
}
#[derive(Debug)]
pub struct FavoriteCourses {
    pub favorite_courses: Vec<FavoriteCourse>,
}
#[derive(Debug)]
pub struct FavoriteCourse {
    pub name: String,
    pub site_id: String,
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
    pub fn get(&mut self) -> Option<&mut SiteContent> {
        self.content_collection.get_mut(0)
    }
}

/// サイトの名前、子リソースを持つ
#[derive(Debug, Deserialize)]
pub struct SiteContent {
    name: String,
    #[serde(rename = "resourceChildren")]
    pub resource_children: Vec<ResourceChild>,
    // #[serde(rename = "mimeType")]
    // mime_type: String,
    // modified: String,
    pub url: String,
}
impl SiteContent {
    pub fn set_url_parent(&mut self) {
        for child in &mut self.resource_children {
            child.url_parent = Some(self.url.clone());
        }
    }
}

/// 親(SiteContent) が持つ子リソースの名前、ファイルタイプ、URL
#[derive(Debug, Deserialize)]
pub struct ResourceChild {
    name: String,
    #[serde(rename = "mimeType")]
    _mime_type: Option<String>,
    // modified: String,
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    pub url: String,
    #[serde(skip)]
    pub url_parent: Option<String>,
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

/// テキストをプロンプト経由で受け取り、file_pathに書き込む。
/// 成功したら `Ok` を返す。
fn get_from_file(prompt: &str, filepath: &str) -> Result<String> {
    let account: String = Input::new()
    .with_prompt(prompt)
    .interact_text()?;
    fs::write(filepath, &account)?;
    Ok(account)
}

/// Get ECS-ID from a user and save it
fn get_account() -> Result<String> {
    get_from_file("Enter your ECS-ID", CONFIG_FILE_PATH)
}
/// Get password from a user and save it
fn get_password() -> Result<String> {
    get_from_file("Enter your password", PASSWORD_FILE_PATH)
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

    let favorite_site_ids: FavoriteSiteIds = client
        .get(format!("{}/portal/favorites/list", BASE_URL))
        .send()
        .await?
        .json::<FavoriteSiteIds>()
        .await?;
    bar.finish_and_clear();

    let mut favorite_courses: Vec<FavoriteCourse> = Vec::new();
    let iter = favorite_site_ids.favorite_site_ids.iter();
    for favorite_site_id in iter {
        let mut site_content_collection = get_site_content(&client, favorite_site_id).await?;
        let site_content = site_content_collection.get().context("No content")?;
        let site_name = &site_content.name;
        favorite_courses.push(FavoriteCourse {
            name: site_name.to_string(),
            site_id: favorite_site_id.to_string(),
        }
        );
    }

    // let favorite_courses: Vec<FavoriteCourse> = favorite_site_ids.favorite_site_ids.iter().map(|favorite_site_id| get_site_content(&client, favorite_site_id).await?).collect();
    Ok(FavoriteCourses {
        favorite_courses: favorite_courses,
    })
}

pub async fn get_site_content(client: &Client, site_id: &String) -> Result<SiteContentCollection> {
    let bar = ProgressBar::new_spinner().with_message("Getting courses...");
    bar.enable_steady_tick(Duration::from_millis(100));

    let site_content = client
        .get(format!("{}/direct/content/resources/{}.json", BASE_URL, site_id))
        .send()
        .await?
        .json::<SiteContentCollection>()
        .await?;
    bar.finish_and_clear();
    Ok(site_content)
}

// 指定されたURLのリソースを取得
pub async fn get_url_content(client: &Client, url: &String) -> Result<SiteContentCollection> {
    let bar = ProgressBar::new_spinner().with_message("Getting courses...");
    bar.enable_steady_tick(Duration::from_millis(100));

    let site_content = client
        .get(format!("{}/direct/content/resources/{}.json", BASE_URL, url))
        .send()
        .await?
        .json::<SiteContentCollection>()
        .await?;
    bar.finish_and_clear();
    Ok(site_content)
}

// 指定されたresource_idの資料を取得
pub async fn get_resource_id_content(client: &Client, resource_id: &str) -> Result<SiteContentCollection> {
    let site_content = client
    .get(format!("{}/direct/content/resources{}.json", BASE_URL, resource_id))

    // .get(format!("{}.json", SEMINAR_URL))
    .send()
    .await?
    .json::<SiteContentCollection>()
    .await?;
    Ok(site_content)
}

pub fn select_site(favorite_courses: &FavoriteCourses) -> Result<usize> {
    let items: Vec<&str> = favorite_courses.favorite_courses.iter().map(|course| course.name.as_str()).collect();
    let selection = FuzzySelect::new()
    .with_prompt("Select course")
    .highlight_matches(true)
    .default(0)
    .items(&items)
    .interact()?;
    // println!("You chose: {}", items[selection]); 
    Ok(selection)
}

pub fn select_child_site(site_content: &SiteContent) -> Result<usize> {
    let items: Vec<&str> = site_content.resource_children.iter().map(|child| child.name.as_str()).collect();
    let selection = FuzzySelect::new()
    .with_prompt("Select item or type to search: ")
    .default(0)
    .items(&items)
    .interact()?;
    Ok(selection)
}

/// 子リソースからひとつ選択する。選択した場合は `Some(番号)`, 戻る場合は`None`を返す
pub fn select_child_site_with_opt(site_content: &SiteContent) -> Result<Option<usize>> {
    let items: Vec<&str> = site_content.resource_children.iter().map(|child| child.name.as_str()).collect();
    let selection = FuzzySelect::new()
    .with_prompt("Select item or type to search: ")
    .default(0)
    .items(&items)
    .interact_opt()?;
    Ok(selection)
}

pub fn open_in_browser(url: &String) -> Result<()> {
    open::that(url)?;
    Ok(())
}