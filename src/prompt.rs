//! Line oriented prompting on stdin/stdout.

use std::io::{self, BufRead, Write};

/// Print `prompt` (without a trailing newline) and read one line of input.
///
/// Returns `Ok(None)` when stdin is closed, so callers can exit cleanly when the
/// input is piped from a file.
pub fn ask(prompt: &str) -> io::Result<Option<String>> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim_end_matches(['\r', '\n']).to_owned()))
}

/// Yes/no question; anything but an explicit "y" counts as no.
pub fn ask_yes(prompt: &str) -> io::Result<bool> {
    Ok(ask(prompt)?.is_some_and(|answer| answer.trim().eq_ignore_ascii_case("y")))
}

/// Ask for a number in `1..=max`, re-asking until the answer parses.
pub fn ask_number(prompt: &str, max: usize) -> io::Result<Option<usize>> {
    loop {
        let Some(answer) = ask(prompt)? else {
            return Ok(None);
        };
        match answer.trim().parse::<usize>() {
            Ok(number) if (1..=max).contains(&number) => return Ok(Some(number)),
            _ => println!("Please enter a number between 1 and {max}."),
        }
    }
}
