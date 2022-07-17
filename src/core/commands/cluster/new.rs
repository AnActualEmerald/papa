use std::fs;

use crate::{
    api::model::Cluster,
    core::config,
    core::{config::ManageMode, Ctx},
};
use anyhow::{Context, Result};

pub(crate) fn new(ctx: &mut Ctx, name: Option<String>) -> Result<()> {
    let target = std::env::current_dir()?.join("cluster.ron");
    let cluster = Cluster::new(name.clone(), target.clone());
    let pretty = ron::ser::to_string_pretty(&cluster, ron::ser::PrettyConfig::new())?;

    fs::write(&target, &pretty).context("Unable to write cluster file")?;
    println!(
        "Created cluster {}",
        if name.is_some() {
            name.as_ref().unwrap()
        } else {
            ""
        }
    );
    ctx.config.mode = ManageMode::Server;
    println!("Set management mode to server");

    config::save_config(ctx.dirs.config_dir(), &ctx.config)?;

    Ok(())
}
