// use std::error::Error;
// use std::fs;
use anyhow::Result;
use dialoguer::{Input, Password};
// use security_framework::passwords::{self, get_generic_password};
// use clap::Parser;

// type MyResult<T> = Result<T, Box<dyn Error>>;

// const SERVICE: &str = "com.apple.network.eap.user.item.wlan.ssid.KUINS-Air";

#[derive(Debug)]
pub struct Credential {
    account: String,
    password: String,
}

pub fn login() -> Result<Credential> {
    let account = get_account()?;
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