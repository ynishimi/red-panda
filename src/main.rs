use red_panda::{get_credential, get_favorite_courses, get_resource_id_content, login, open_in_browser, select_child_site, select_site, ResourceChild};
use tokio;
use anyhow::{Result, Context};
use reqwest::Client;

const SEMINAR_RESOURCE_ID: &str = "/group/ae7eb08f-5eab-41d2-a8d8-229aac826b97/2024年度_定例セミナー _Weekly Seminar 2024_/";

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
    // let favorite_courses = get_favorite_courses(&client).await?;
    // let course_selection = select_site(&favorite_courses)?;
    // let site_content_collection = get_site_content(&client, &favorite_courses.favorite_courses[course_selection].site_id).await?;
    // let site_content = site_content_collection
    //     .get()
    //     .context("Content not available")?;
    
    // リソースを取得
    let mut seminar_content_collection = get_resource_id_content(&client, SEMINAR_RESOURCE_ID).await?;
    let seminar_content = seminar_content_collection
        .get()
        .context("Content not available")?;
    // 子にurl_parentを追加
    seminar_content.set_url_parent();

    // 子から1つ選ぶ
    let child_number = select_child_site(&seminar_content)?;
    let child_site = &seminar_content.resource_children[child_number];

    // 子以下のリソースを取得
    let mut content_collection = get_resource_id_content(&client, &child_site.resource_id).await?;
    let content = content_collection
    .get()
    .context("Content not available")?;
    // 一つ選ぶ
    let content_number = select_child_site(&content)?;
    let site = &content.resource_children[content_number];


    open_in_browser(&site.url)?;


    // println!("{:?}", favorite_courses.favorite_site_ids);
    

    // for site_id in favorite_courses.favorite_site_ids {
    //     // if let Ok(res) = get_site_content(&client, site_id).await {

    //         // println!("{}", res);
    //     // }
    // }
    Ok(())
}
