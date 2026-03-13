mod ping;
mod info;
mod support;

use crate::{Data, Error};

pub fn all_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        ping::ping(),
        info::info(),
        support::support(),
    ]
}
