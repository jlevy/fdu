use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let command = acorn::Command::parse();
    let report = acorn::run(command)?;
    print!("{report}");
    Ok(())
}
