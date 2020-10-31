use std::io;
// bring flush() into scope
use std::io::Write;


pub fn read_stdin(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();

    let mut reply = String::new();
    io::stdin()
        .read_line(&mut reply)
        .expect("Could not read input.");
    reply.retain(|c| !c.is_whitespace());

    reply
}
