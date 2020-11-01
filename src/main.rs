use std::{process::Command, collections::HashMap};

use bitwarden_service::{get_by_name, auth::get_token};
use clap::Clap;

mod models;
mod env;

/// Execute a command with environment variables from .env files or
/// Bitwarden secure notes
#[derive(Clap)]
#[clap(version = "0.2.0", author = "Xyder <xyder@dsider.org>")]
struct Opts {
    /// Load env vars from an .env file
    #[clap(short, long, default_value = "./.env")]
    file: String,

    /// If this is present all env sources will be merged, first loading Bitwarden,
    /// then the .env file.
    #[clap(short, long)]
    cumulative: bool,

    /// Load env vars from one or more Bitwarden secure notes. If multiple notes
    /// containing the same `bitwarden-name` are found, they will be merged in
    /// alphabetical order and identical fields overwritten.
    /// (Example:
    /// Having two notes named "MyApp environment A" and "MyApp environment B"
    /// will cause any identical fields to be taken from "MyApp environment B")
    #[clap(short, long)]
    bitwarden_name: Option<String>,

    /// The shell to run this command in.
    #[clap(short, long, default_value = "/bin/sh")]
    shell: String,
    command: Vec<String>
}

#[tokio::main]
async fn main() -> models::BoxedResult<()> {
    let opts = Opts::parse();

    if opts.command.len() == 0 {
        println!("Error: No command supplied.");
        std::process::exit(2);

    }

    let bw_envs = match opts.bitwarden_name {
        Some(v) => {
            let token = get_token().await.unwrap();
            get_by_name(&v, &token).await.unwrap()
        },
        None => HashMap::new()
    };
    let file_envs = env::get_env_vars(&opts.file);

    if file_envs.is_err() {
        println!("No env file found.")
    }

    if bw_envs.len() == 0 {
        println!("No BW envs loaded.")
    }

    let file_envs = file_envs.unwrap_or(HashMap::new());

    let envs = if opts.cumulative {
        bw_envs.into_iter().chain(file_envs).collect()
    } else {
        if bw_envs.len() != 0 {
            bw_envs
        } else {
            file_envs
        }
    };

    let output = Command::new(opts.shell)
        .arg("-c")
        .arg(opts.command.join(" "))
        .envs(&envs)
        // runs the command with stdin, stdout and stderr inherited from the parent
        // alternatively can be run with Stdio::inherit
        .status();

    if output.is_err() {
        let e = output.unwrap_err();
        println!("{:?} error: {}", e.kind(), e);
        std::process::exit(1);
    }
    Ok(())
}
