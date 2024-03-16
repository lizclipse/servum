#![warn(clippy::pedantic)]

use color_eyre::eyre;

mod cli;
mod config;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    Ok(())
}
