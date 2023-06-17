mod cli;
mod constants;
mod errors;
mod utils;
use clap::{Args, Parser, Subcommand};
use color_eyre::eyre;
use std::path::PathBuf;
use utils::find_package_root;

pub fn start_cli() -> eyre::Result<()> {
    let Halo2Cli {
        command,
        mut config,
    } = Halo2Cli::parse();

    config.program_dir = find_package_root(&config.program_dir)?;

    match command {
        Halo2Command::Prove(args) => cli::prove_cmd::run(args, config),
        Halo2Command::Verify(args) => cli::verify_cmd::run(args, config),
        Halo2Command::Contract(args) => cli::contract::run(args, config),
        Halo2Command::Gates(args) => cli::gates_cmd::run(args, config),
        Halo2Command::ProveAndVerify(args) => cli::prove_and_verify_cmd::run(args, config),
    }?;

    Ok(())
}

#[derive(Parser, Debug)]
#[command(name="halo2-kzg", author, about, long_about = None)]
struct Halo2Cli {
    #[command(subcommand)]
    command: Halo2Command,

    #[clap(flatten)]
    config: Halo2Config,
}

#[non_exhaustive]
#[derive(Args, Clone, Debug)]
pub(crate) struct Halo2Config {
    #[arg(short, long, hide=true, default_value_os_t = std::env::current_dir().unwrap())]
    program_dir: PathBuf,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum Halo2Command {
    Contract(cli::contract::ContractCommand),
    Prove(cli::prove_cmd::ProveCommand),
    Verify(cli::verify_cmd::VerifyCommand),
    Gates(cli::gates_cmd::GatesCommand),
    ProveAndVerify(cli::prove_and_verify_cmd::ProveAndVerifyCommand),
}
