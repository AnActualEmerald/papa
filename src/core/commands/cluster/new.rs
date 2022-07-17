use std::{collections::HashMap, fs};

use crate::api::model::Cluster;
use anyhow::{Context, Result};

pub(crate) fn new(name: Option<String>) -> Result<()> {
    let cluster = Cluster {
        name,
        members: HashMap::new(),
    };
    let target = std::env::current_dir()?.join("cluster.ron");
    let pretty = ron::ser::to_string_pretty(&cluster, ron::ser::PrettyConfig::new())?;

    fs::write(&target, &pretty).context("Unable to write cluster file")?;

    Ok(())
}
