use anyhow::Context;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
mod commands;
mod object;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    CatFile {
        #[arg(short)]
        pretty_print: bool,
        object: String,
    },
    HashObject {
        #[arg(short)]
        write: bool,
        file: PathBuf,
    },
    LsTree {
        #[arg(long)]
        name_only: bool,
        tree_hash: String,
    },
    WriteTree,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::invoke().context("invoke Init"),
        Commands::CatFile {
            pretty_print,
            object,
        } => commands::cat_file::invoke(pretty_print, &object)
            .context("invoke cat_file")
            .context("invoke CatFile"),

        Commands::HashObject { write, file } => {
            commands::hash_object::invoke(write, &file).context("invoke HashObject")
        }

        Commands::LsTree {
            name_only,
            tree_hash,
        } => commands::ls_tree::invoke(name_only, &tree_hash),

        Commands::WriteTree => commands::write_tree::invoke(),
    }
}
