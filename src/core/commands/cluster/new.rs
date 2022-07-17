use std::fs;

use crate::api::model::Cluster;
use anyhow::{Context, Result};

pub(crate) fn new(name: Option<String>) -> Result<()> {
    let target = std::env::current_dir()?.join("cluster.ron");
    let cluster = Cluster::new(name, target.clone());
    let pretty = ron::ser::to_string_pretty(&cluster, ron::ser::PrettyConfig::new())?;

    fs::write(&target, &pretty).context("Unable to write cluster file")?;

    Ok(())
}
