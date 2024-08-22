use red_panda::{get_credential, get_favorite_courses, get_site_content, login, open_in_browser, select_child_site, select_site};
use tokio;
use anyhow::{Result, Context};
use reqwest::Client;

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
    let favorite_courses = get_favorite_courses(&client).await?;
    let course_selection = select_site(&favorite_courses)?;
    let site_content = get_site_content(&client, &favorite_courses.favorite_courses[course_selection].site_id).await?;
    let site_content = site_content
        .get()
        .context("Content not available")?;
    let child_selection = site_content;
    let selection_children = select_child_site(&child_selection)?;
    let resource_children = &child_selection.resource_children;

    open_in_browser(&resource_children[selection_children].url)?;


    // println!("{:?}", favorite_courses.favorite_site_ids);
    

    // for site_id in favorite_courses.favorite_site_ids {
    //     // if let Ok(res) = get_site_content(&client, site_id).await {

    //         // println!("{}", res);
    //     // }
    // }
    Ok(())
}
