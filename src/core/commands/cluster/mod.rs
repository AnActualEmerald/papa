mod add;
mod new;
use std::path::PathBuf;

use add::add;
use anyhow::Result;
use clap::Subcommand;
use new::new;

use crate::core::Ctx;

#[derive(Subcommand)]
pub(crate) enum WsCommands {
    New { name: Option<String> },
    Add { name: String, path: PathBuf },
    Remove {},
}

///Handle cluster subcommands
pub(crate) fn cluster(ctx: &mut Ctx, command: WsCommands) -> Result<()> {
    match command {
        WsCommands::New { name } => new(name),
        WsCommands::Add { name, path } => add(ctx, name, path),
        WsCommands::Remove {} => Ok(()),
    }
}
