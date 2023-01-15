use crate::config::DIRS;
use crate::traits::RemoteIndex;
use crate::{
    config::{write_config, CONFIG},
    model::ModName,
    NstarCommands,
};
use anyhow::{anyhow, Result};
use owo_colors::OwoColorize;
use thermite::prelude::*;
use tracing::{debug, instrument};

pub async fn northstar(commands: &NstarCommands) -> Result<()> {
    match commands {
        NstarCommands::Init { force } => init_ns(force).await?,
        NstarCommands::Update {} => todo!(),
    }

    Ok(())
}

#[instrument]
async fn init_ns(force: &bool) -> Result<()> {
    let Some(titanfall) = titanfall() else {
        return Err(anyhow!("Unable to locate Titanfall 2 in Steam libraries"));
    };

    debug!("Installing N* to '{}'", titanfall.display());

    //try to detect existing installation
    if !force {
        if titanfall.join("NorthstarLauncher.exe").try_exists()? {
            println!("Found an existing Northstar installation, updating config!");
            let mut new_config = CONFIG.clone();
            new_config.set_install_dir(titanfall.join("R2Northstar").join("mods"));
            write_config(&new_config)?;
            return Ok(());
        }
    }

    let index = get_package_index().await?;
    let nsmod = index
        .get_mod(&ModName::new("northstar", "Northstar", None))
        .ok_or(anyhow!("Couldn't find Northstar in the package index"))?;

    println!("Downloading Northstar version {}...", nsmod.latest.bold());

    let nsfile = download_file(
        &nsmod
            .get_latest()
            .expect("N* mod missing latest version")
            .url,
        DIRS.cache_dir()
            .join(format!("{}-{}.zip", nsmod.name, nsmod.latest)),
    )
    .await?;

    println!("Installing Northstar...");
    install_northstar(&nsfile, &titanfall).await?;
    println!("Done!");

    let mut new_config = CONFIG.clone();
    new_config.set_install_dir(titanfall.join("R2Northstar").join("mods"));
    write_config(&new_config)?;

    Ok(())
}
