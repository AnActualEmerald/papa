mod add;
mod list;
mod new;
mod remove;
use std::path::PathBuf;

use add::add;
use anyhow::Result;
use clap::Subcommand;
use list::list;
use new::new;
use remove::remove;

use crate::core::Ctx;

#[derive(Subcommand)]
pub(crate) enum WsCommands {
    ///Create a new cluster
    #[clap(alias("n"))]
    New { name: Option<String> },
    ///Add a folder to an existing cluster
    #[clap(alias("a"))]
    Add {
        #[clap(short, long)]
        name: Option<String>,
        path: PathBuf,
    },
    ///Remove a folder from a cluster
    #[clap(alias("r"))]
    Remove { name: String },
    ///List the members of a cluster
    #[clap(alias("l"), alias("ls"))]
    List {},
}

///Handle cluster subcommands
pub(crate) fn cluster(ctx: &mut Ctx, command: WsCommands) -> Result<()> {
    match command {
        WsCommands::New { name } => new(ctx, name),
        WsCommands::Add { name, path } => add(ctx, name, path),
        WsCommands::Remove { name } => remove(ctx, name),
        WsCommands::List {} => list(ctx),
    }
}
