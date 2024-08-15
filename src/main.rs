
fn main() {
    // red_panda::get_password("a0210791");
    let credential = red_panda::login();
    println!("{:?}", credential);
}

