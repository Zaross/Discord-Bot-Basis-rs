mod admin;
mod help;
mod info;
mod ping;
mod serverinfo;
mod support;
mod userinfo;

use crate::{Data, Error};

pub fn all_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        ping::ping(),
        info::info(),
        support::support(),
        help::help(),
        userinfo::userinfo(),
        serverinfo::serverinfo(),
        admin::status(),
        admin::sql(),
    ]
}
