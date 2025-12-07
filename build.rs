use eyre::Result;
fn main() -> Result<()> {
    shadow_rs::ShadowBuilder::builder().build()?;
    Ok(())
}
