//! Saving a [`Post`] as a training sample on disk.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use reqwest::Url;

use crate::boorus::Http;
use crate::post::{ContentType, Post};

/// Download the post's image into `dir` and write the matching caption.
///
/// The caption is only written once the image is on disk, so a failed download
/// never leaves a caption without its image behind.
pub fn save(post: &Post, dir: &Path, content_type: ContentType, http: &Http) -> Result<()> {
    let url = Url::parse(&post.file_url)
        .with_context(|| format!("invalid image URL {:?}", post.file_url))?;
    let response = http.get(url)?;
    if !response.status().is_success() {
        println!("Failed to download image.");
        return Ok(());
    }
    let image = response.bytes().context("reading the image body")?;

    let stem = dir.join(&post.id);
    let image_path = stem.with_extension(&post.file_ext);
    fs::write(&image_path, &image).with_context(|| format!("writing {}", image_path.display()))?;
    println!("Image downloaded successfully.");

    let caption_path = stem.with_extension("txt");
    fs::write(&caption_path, post.training_caption(content_type))
        .with_context(|| format!("writing {}", caption_path.display()))?;
    println!("saved tags");

    Ok(())
}
