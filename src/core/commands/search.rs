use crate::traits::RemoteIndex;
use anyhow::Result;
use owo_colors::OwoColorize;
use thermite::prelude::*;
use tracing::debug;

pub async fn search(term: &[String]) -> Result<()> {
    let index = get_package_index().await?;
    let term = term.join("");
    debug!("Searching for term '{}'", term.bold());

    let res = index.search(&term);
    if res.len() == 0 {
        println!("No mods matched '{}'", term.bold());
        return Ok(());
    }

    println!("Found packages: ");
    for m in res {
        let latest = m.get_latest().unwrap();
        let desc = latest.desc.clone();
        println!(
            " {}.{}@{} - {}\n    {}",
            m.author.bright_blue(),
            m.name.bright_blue(),
            m.latest.bright_blue(),
            latest.file_size_string().bright_yellow(),
            desc
        );
    }

    Ok(())
}
