// use std::error::Error;
use std::str;
use std::fs;
use anyhow::Result;
use dialoguer::{Input, Password};
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
}

pub fn login() -> Result<Credential> {
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

// pub fn get_password(account: &str) -> MyResult<()> {
//     let pass = get_generic_password(SERVICE, account)?;
//     // save account
//     fs::write("~/.config/red-panda/config.yml", account)?;
//     let pass = str::from_utf8(&pass)?;
//     println!("{}", pass);
//     Ok(())
// }