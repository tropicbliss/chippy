use anyhow::Result;
use chippy::CPU;
use clap::Parser;
use std::path::PathBuf;

#[macroquad::main("Chippy")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut cpu = CPU::new().await;
    cpu.load(args.rom)?;
    cpu.run(args.debug).await?;
    Ok(())
}

#[derive(Debug, Parser)]
#[clap(version, about)]
struct Args {
    /// Path to the ROM binary
    rom: PathBuf,

    /// Enable debug menu
    #[clap(short, long)]
    debug: bool,
}
