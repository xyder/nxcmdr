use std::io;
// bring flush() into scope
use std::io::Write;

use anyhow::{Context, Result};


fn read_stdin(msg: &str) -> Result<String> {
    print!("{}", msg);
    io::stdout().flush().unwrap_or(());

    let mut reply = String::new();
    io::stdin()
        .read_line(&mut reply)
        .context("Could not read input")?;

    reply.retain(|c| !c.is_whitespace());

    Ok(reply)
}


pub fn read_from_stdin(initial: &Option<String>, message: &str, secure: bool) -> Result<String> {
    let initial = match initial {
        Some(v) => v.clone(),
        None => "".to_string()
    };
    let initial = initial.trim();

    let output = match initial {
        "" => match secure {
            true => rpassword::prompt_password_stdout(message)
                .context("Could not read hidden input")?,
            false => read_stdin(message)?
        },
        v => v.to_string()
    };

    Ok(output.trim().to_string())
}
