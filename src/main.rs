use red_panda::get_login_token;
use tokio;

#[tokio::main]
async fn main() {
    match get_login_token().await {
        Ok(_) => println!(),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

}
