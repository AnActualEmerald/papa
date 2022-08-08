use crate::{api::model::LocalIndex, core::Ctx};
use anyhow::Result;

pub fn list(ctx: &Ctx, global: bool, all: bool) -> Result<()> {
    let do_list = |target, global| -> Result<()> {
        let index = LocalIndex::load(target)?;
        let msg = if global {
            "Global mods:"
        } else {
            "Local mods:"
        };
        println!("{}", msg);
        if !index.mods.is_empty() {
            index.mods.iter().for_each(|(_, m)| {
                let disabled = if !m.any_disabled() || m.mods.len() > 1 {
                    ""
                } else {
                    "[disabled]"
                };
                println!(
                    "  \x1b[92m{}@{}\x1b[0m {}",
                    m.package_name, m.version, disabled
                );
                if m.mods.len() > 1 {
                    for (i, e) in m.mods.iter().enumerate() {
                        let character = if i + 1 < m.mods.len() { "├" } else { "└" };
                        let disabled = if e.disabled() { "[disabled]" } else { "" };
                        println!(
                            "    \x1b[92m{}─\x1b[0m \x1b[0;96m{}\x1b[0m {}",
                            character, e.name, disabled
                        );
                    }
                }
            });
        } else {
            println!("  No mods currently installed");
        }
        println!();
        if !index.linked.is_empty() {
            println!("Linked mods:");
            index
                .linked
                .iter()
                .for_each(|(_, m)| println!("  \x1b[92m{}@{}\x1b[0m", m.package_name, m.version));
            println!();
        }

        Ok(())
    };

    if !all {
        let target = if global {
            ctx.dirs.data_local_dir()
        } else {
            ctx.config.mod_dir()
        };

        do_list(target, global)
    } else {
        do_list(ctx.config.mod_dir(), false)?;
        do_list(ctx.dirs.data_local_dir(), true)?;
        Ok(())
    }
}
