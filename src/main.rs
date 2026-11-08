//! Interactive CLI: pick a folder, pick a booru, then convert post ids into
//! image/caption pairs until the input ends.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;

use booru2trainingdata::boorus::{
    Booru, Credentials, DANBOORU_URL, Danbooru, GELBOORU_OLD_URL, GELBOORU_URL, Gelbooru, Http,
};
use booru2trainingdata::post::ContentType;
use booru2trainingdata::prompt;
use booru2trainingdata::{download, paths};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let Some(save_dir) = save_directory()? else {
        return Ok(());
    };
    println!("Saving images to {}", save_dir.display());

    println!("What Booru service would you like to use?");
    println!("1. Danbooru(What most anime models are already trained on)");
    println!(
        "2. Gelbooru(Tags can be diffrent so might not be as good, however has less censorship without Gold account)"
    );
    println!("3. Gelbooru old(same API as rule34 xxx as it seems to be on older version)");

    let Some(choice) = prompt::ask_number("Pick a number 1..3: ", 3)? else {
        return Ok(());
    };

    let http = Http::new()?;
    let booru: Box<dyn Booru> = match choice {
        1 => Box::new(Danbooru::new(DANBOORU_URL, &http)?),
        2 => {
            let base = base_url(
                "Would you like to change the Gelbooru URL from the default(https://gelbooru.com)? (Y/n): ",
                GELBOORU_URL,
            )?;
            let api_key = prompt::ask("Please enter your Gelbooru API Key: ")?.unwrap_or_default();
            let user_id = prompt::ask("Please enter your Gelbooru User ID: ")?.unwrap_or_default();
            Box::new(Gelbooru::new(
                &base,
                Some(Credentials { api_key, user_id }),
                &http,
            )?)
        }
        _ => {
            let base = base_url(
                "Would you like to change the Gelbooru URL from the default(https://rule34.xxx/)? (Y/n): ",
                GELBOORU_OLD_URL,
            )?;
            Box::new(Gelbooru::new(&base, None, &http)?)
        }
    };

    let content_type = if prompt::ask_yes("Are your images NSFW? (Y/n): ")? {
        ContentType::Nsfw
    } else {
        ContentType::Sfw
    };

    while let Some(id) = prompt::ask(&format!("Enter a image ID for {}: ", booru.label()))? {
        let id = id.trim();
        if id.is_empty() {
            continue;
        }

        match booru.fetch(id) {
            Ok(Some(post)) => {
                if let Err(error) = download::save(&post, &save_dir, content_type, &http) {
                    println!("Failed to download image.");
                    eprintln!("{error:#}");
                }
            }
            Ok(None) => println!("Failed to retrieve post data."),
            Err(error) => {
                println!("Failed to retrieve post data.");
                eprintln!("{error:#}");
            }
        }
    }

    Ok(())
}

/// Ask where to save images, creating the folder when the user agrees.
///
/// Returns `None` when the input ends, which ends the program quietly.
fn save_directory() -> Result<Option<PathBuf>> {
    let Some(answer) = prompt::ask("Pick a directory to save images to: ")? else {
        return Ok(None);
    };
    // Expand `~` here: input typed at a prompt does not go through a shell.
    let dir = paths::expand_tilde(&answer);

    if dir.exists() {
        return Ok(Some(dir));
    }

    let create = prompt::ask_yes(&format!(
        "The directory '{}' does not exist. Would you like to create it? (Y/n): ",
        dir.display()
    ))?;
    if !create {
        println!("Directory required. Exiting.");
        std::process::exit(1);
    }

    if let Err(error) = fs::create_dir_all(&dir) {
        eprintln!("Error creating directory: {error}");
        std::process::exit(1);
    }
    println!("Created directory: {}", dir.display());

    Ok(Some(dir))
}

/// Base URL for a Gelbooru style service: the default unless the user overrides it.
fn base_url(prompt: &str, default: &str) -> Result<String> {
    if !prompt::ask_yes(prompt)? {
        return Ok(default.to_owned());
    }

    let Some(answer) = prompt::ask("Please type a URL: ")? else {
        return Ok(default.to_owned());
    };
    let answer = answer.trim();
    if answer.is_empty() {
        return Ok(default.to_owned());
    }

    Ok(answer.to_owned())
}
