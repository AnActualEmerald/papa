use crate::{
    api::model::Mod,
    core::{utils, Ctx},
};

use anyhow::Result;

pub(crate) async fn search(ctx: &Ctx, term: Vec<String>) -> Result<()> {
    let index = utils::update_index(ctx.config.mod_dir(), &ctx.global_target).await;

    let print = |f: &Mod| {
        println!(
            " \x1b[92m{}@{}\x1b[0m   [{}]{}\n\n    {}",
            f.name,
            f.version,
            f.file_size_string(),
            if f.installed { "[installed]" } else { "" },
            f.desc
        );
        println!();
    };

    println!("Searching...");
    println!();
    if !term.is_empty() {
        index
            .iter()
            .filter(|f| {
                //TODO: Use better method to match strings
                term.iter().any(|e| {
                    f.name.to_lowercase().contains(&e.to_lowercase())
                        || f.desc.to_lowercase().contains(&e.to_lowercase())
                })
            })
            .for_each(print);
    } else {
        index.iter().for_each(print)
    }
    Ok(())
}
