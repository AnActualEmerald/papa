use std::path::PathBuf;

use anyhow::Result;

use crate::core::Ctx;

pub(super) fn add(ctx: &mut Ctx, name: Option<String>, path: PathBuf) -> Result<()> {
    if let Some(c) = &mut ctx.cluster {
        let name = name.unwrap_or_else(|| path.file_name().unwrap().to_str().unwrap().to_string());
        c.members.insert(name.clone(), path.canonicalize()?);
        c.save()?;
        println!(
            "Added {}(at {}) to cluster {}",
            name,
            path.display(),
            if c.name.is_some() {
                c.name.as_ref().unwrap()
            } else {
                ""
            }
        );
    } else {
        println!("There is no cluster to add to!");
    }
    Ok(())
}
