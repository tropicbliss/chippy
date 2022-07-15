use anyhow::Result;
use chippy::CPU;
use clap::Parser;
use macroquad::rand::{gen_range, srand};
use std::path::PathBuf;

#[macroquad::main("Chippy")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut cpu = CPU::new().await;
    cpu.load(args.rom.to_str().unwrap()).await?;
    cpu.run(args.debug).await?;
    Ok(())
}

#[derive(Debug, Parser)]
#[clap(version, about)]
struct Args {
    /// Path to the ROM binary
    rom: PathBuf,

    /// Enable debug menu (spamming this increases verbosity)
    #[clap(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}
