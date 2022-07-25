use crate::core::Ctx;
use anyhow::Result;

pub(crate) fn remove(ctx: &mut Ctx, name: String) -> Result<()> {
    if let Some(c) = ctx.cluster.as_mut() {
        if !c.members.contains_key(&name) {
            println!("Couldn't find member with name '{}'", name);
            return Ok(());
        }

        c.members.remove(&name);
        println!("Removed '{}' from cluster", name);
    } else {
        println!("There is no cluster set up!");
    }

    Ok(())
}
