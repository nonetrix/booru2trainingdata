//! Download booru posts and save them as image/caption pairs suitable for
//! fine-tuning anime txt2img models.
//!
//! The crate is split into the pieces the interactive binary needs:
//!
//! * [`boorus`] talks to Danbooru and Gelbooru (current and legacy APIs).
//! * [`post`] holds a post and renders the caption written next to the image.
//! * [`download`] turns a post into the two files on disk.
//! * [`paths`] expands what the user typed into a usable directory path.
//! * [`prompt`] provides the small amount of stdin/stdout prompting the CLI uses.

pub mod boorus;
pub mod download;
pub mod paths;
pub mod post;
pub mod prompt;
