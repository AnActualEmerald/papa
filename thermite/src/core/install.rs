use crate::{
    core::actions,
    error::ThermiteError,
    model::{LocalIndex, Mod},
};

use log::{debug, error, trace};

use super::Ctx;

/// Download and install mod(s) to the specified target. Will check the cache before downloading if configured.
/// # Params
/// * `ctx` - The current context
/// * `target` - The index to install to
/// * `mods` - The mods to install
/// * `force` - Ignore non-fatal errors
pub async fn install(
    ctx: &mut Ctx,
    target: &mut LocalIndex,
    mods: Vec<Mod>,
    _force: bool,
    cache: bool,
) -> Result<(), ThermiteError> {
    let mut downloaded = vec![];
    for base in mods {
        let name = &base.name;
        let path = ctx
            .dirs
            .cache_dir()
            .join(format!("{}_{}.zip", name, base.version));

        //would love to use this in the same if as the let but it's unstable so...
        if cache {
            if let Some(f) = ctx.cache.check(&path) {
                debug!("Using cached version of {}", name);
                downloaded.push(f);
                continue;
            }
        }
        match actions::download_file(&base.url, path).await {
            Ok(f) => downloaded.push(f),
            Err(e) => error!("{}", e),
        }
    }

    trace!(
        "Extracting mod{} to {}",
        if downloaded.len() > 1 { "s" } else { "" },
        target.path().display()
    );
    for e in downloaded
        .iter()
        .map(|f| -> Result<(), ThermiteError> {
            let pkg = actions::install_mod(f, target.parent_dir().as_ref())?;
            target.mods.insert(pkg.package_name.clone(), pkg.clone());
            ctx.cache.clean(&pkg.package_name, &pkg.version)?;
            Ok(())
        })
        .filter(|f| f.is_err())
    {
        error!("Encountered errors while installing mods:");
        error!("{:#?}", e.unwrap_err());
    }
    Ok(())
}
