use std::{process::Command, collections::HashMap};

use clap::Clap;
use anyhow::Result;

use bitwarden_service::{get_by_name, auth::get_token};

mod env;

/// Execute a command with environment variables from .env files or
/// Bitwarden secure notes
#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = "xyder <xyder@dsider.org>")]
struct Opts {
    /// Load env vars from an .env file
    #[clap(short, long, default_value = "./.env")]
    file: String,

    /// If this is present all env sources will be merged, first loading Bitwarden,
    /// then the .env file.
    #[clap(short, long)]
    cumulative: bool,

    /// If this is present, no output will be printed (except for when printing environment variables, if needed)
    #[clap(short)]
    quiet: bool,

    /// If this is present, the environment variables will be printed to stdout and the command will not be executed
    #[clap(short, long)]
    list: bool,

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

    /// the command to run
    command: Vec<String>,
}

fn bw_get_by_name(name: &str, quiet: bool) -> Result<HashMap<String, String>> {
    let token = get_token(quiet)?;
    get_by_name(name, &token).map_err(|e|e.into())
}

fn main() {
    let opts = Opts::parse();
    let quiet = opts.quiet || opts.list;

    if !opts.list && opts.command.len() == 0 {
        if !quiet {
            println!("Error: No command supplied.");
        }
        std::process::exit(2);

    }

    let bw_envs = match opts.bitwarden_name {
        Some(v) => {
            bw_get_by_name(&v, quiet).unwrap_or_else(|error| {
                println!("{:#}", error);
                std::process::exit(2);
            })
        },
        None => HashMap::new()
    };
    let file_envs = env::get_env_vars(&opts.file);

    if file_envs.is_err() {
        if !quiet {
            println!("No env file found.")
        }
    }

    if bw_envs.len() == 0 {
        if !quiet {
            println!("No BW envs loaded.")
        }
    }

    let file_envs = file_envs.unwrap_or(HashMap::new());

    let envs = if opts.cumulative {
        let mut res: HashMap<String, String> = HashMap::new();
        res.extend(bw_envs);
        res.extend(file_envs);
        res
    } else {
        if bw_envs.len() != 0 {
            bw_envs
        } else {
            file_envs
        }
    };

    if !quiet {
        println!("Loaded {} environment variables.", envs.len());
    }

    if opts.list {
        for (env_key, env_val) in envs {
            println!("{}='{}'", env_key, env_val);
        }

        return
    }

    let output = Command::new(opts.shell)
        .arg("-c")
        .arg(opts.command.join(" "))
        .envs(&envs)
        // runs the command with stdin, stdout and stderr inherited from the parent
        // alternatively can be run with Stdio::inherit
        .status();

    if output.is_err() {
        let e = output.unwrap_err();
        if !quiet {
            println!("{:?} error: {}", e.kind(), e);
        }
        std::process::exit(1);
    }
}
