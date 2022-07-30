use anyhow::{anyhow, Result};
use clap::Subcommand;

use crate::{api::model::Profile, core::Ctx};

#[derive(Subcommand)]
pub enum ProfCommands {
    ///Create a new mod profile
    Create { name: String },
    ///Add a mod to a profile
    Add {
        name: String,
        ///Profile to modify to. Defaults to the current profile
        #[clap(long, short)]
        profile: Option<String>,
    },
    ///Remove a mod from the a profile
    Remove {
        name: String,
        ///Profile to modify. Defaults to the current profile
        #[clap(long, short)]
        profile: Option<String>,
    },
}

pub fn profile(ctx: &mut Ctx, command: ProfCommands) -> Result<()> {
    match command {
        ProfCommands::Create { name } => {
            Profile::get(ctx.dirs.config_dir(), &name)?;
            println!("Created profile \"{}\"", name);
        }
        ProfCommands::Add { name, profile } => add_mod(ctx, name, profile)?,
        _ => {}
    }

    Ok(())
}

//Add a mod to the target profile
fn add_mod(ctx: &mut Ctx, name: String, pname: Option<String>) -> Result<()> {
    let mut target = if let Some(p) = pname {
        Profile::get(ctx.dirs.config_dir(), &p)?
    } else {
        Profile::get(ctx.dirs.config_dir(), &ctx.config.profile)?
    };

    if let Some(m) = ctx
        .local_installed
        .mods
        .iter()
        .find(|e| e.package_name.to_lowercase() == name.to_lowercase())
    {
        target.mods.insert(m.clone());
    } else if let Some(m) = ctx
        .global_installed
        .mods
        .iter()
        .find(|e| e.package_name.to_lowercase() == name.to_lowercase())
    {
        target.mods.insert(m.clone());
    } else {
        println!("Unable to find mod '{}'", name);
        return Ok(());
    }

    println!("Added mod '{}' to profile {}", name, target.name);

    Ok(())
}
