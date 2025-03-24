use crate::traits::Index;
use anyhow::Result;
use owo_colors::OwoColorize;
use textwrap::Options;
use thermite::prelude::*;
use tracing::debug;

pub fn search(term: &[String]) -> Result<()> {
    let index = get_package_index()?;
    let term = term.join("");
    debug!("Searching for term '{}'", term.bold());

    let res = index.search(&term);
    if res.is_empty() {
        println!("No mods matched '{}'", term.bold());
        return Ok(());
    }

    println!("Found packages: ");
    for m in res {
        let latest = m.get_latest().unwrap();
        // ensures that descriptions with newline characters don't break the formatting
        let desc = {
            let opt = Options::with_termwidth();
            let tmp = textwrap::fill(&latest.desc, opt);
            textwrap::indent(&tmp, "    ")
        };

        println!(
            " {}.{}@{} - {}\n{}",
            m.author.bright_blue(),
            m.name.bright_blue(),
            m.latest.bright_blue(),
            latest.file_size_string().bright_yellow(),
            desc
        );
    }

    Ok(())
}
