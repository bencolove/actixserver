use {
    std::time::Duration,
    futures::future::{ Future, ready },
    tokio::time::{
        delay_for,
    },
};

#[derive(Debug)]
pub enum UserAuth {
    None,
    UserInfo {
        name: String,
        token: String,
    },
}

pub fn verify_token(token: &str) -> UserAuth {
    
    // delay_for(Duration::from_secs(1)).await;

    if (token.is_empty()) {
        println!("missing token");
    } else {
        println!("token is {}", token);
    }

    UserAuth::UserInfo {
        name: String::from("roger"),
        token: String::from(token),
    }
}
