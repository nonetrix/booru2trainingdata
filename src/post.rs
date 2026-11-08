//! The subset of a booru post that a training sample needs, and its caption.

use std::fmt;

/// Whether the images being saved are NSFW; the value ends up in the caption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Sfw,
    Nsfw,
}

impl ContentType {
    /// Map a yes/no answer to a content type. Anything but an explicit "y" is
    /// treated as safe, matching the original script.
    pub fn from_answer(answer: &str) -> Self {
        if answer.trim().eq_ignore_ascii_case("y") {
            Self::Nsfw
        } else {
            Self::Sfw
        }
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Sfw => "sfw",
            Self::Nsfw => "nsfw",
        })
    }
}

/// A booru post reduced to what is needed to write a training sample.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    /// Post id as reported by the API; also the base name of the saved files.
    pub id: String,
    /// Direct link to the full size image.
    pub file_url: String,
    /// Image extension such as `png`.
    pub file_ext: String,
    /// Space separated tags, in the form the API returns them.
    pub tags: String,
}

impl Post {
    /// Extension of the file behind `file_url`.
    ///
    /// Deliberately naive (everything after the last dot) so file names match
    /// what the original Python script produced.
    pub fn ext_from_url(file_url: &str) -> String {
        file_url.rsplit('.').next().unwrap_or_default().to_owned()
    }

    /// Caption written to `<id>.txt`, in the tag style txt2img models expect:
    /// comma separated, underscores turned into spaces, parentheses escaped.
    pub fn training_caption(&self, content_type: ContentType) -> String {
        let commas = self.tags.split_whitespace().collect::<Vec<_>>().join(", ");
        let readable = commas.replace('_', " ");
        let escaped = readable.replace('(', "\\(").replace(')', "\\)");
        format!("masterpiece, highest quality, {content_type}, {escaped}")
    }
}

#[cfg(test)]
mod tests {
    use super::{ContentType, Post};

    fn post(tags: &str) -> Post {
        Post {
            id: "1".to_owned(),
            file_url: "https://example.test/1.png".to_owned(),
            file_ext: "png".to_owned(),
            tags: tags.to_owned(),
        }
    }

    #[test]
    fn caption_is_comma_separated_and_readable() {
        let caption = post("1girl solo red_hair").training_caption(ContentType::Nsfw);
        assert_eq!(
            caption,
            "masterpiece, highest quality, nsfw, 1girl, solo, red hair"
        );
    }

    #[test]
    fn caption_escapes_parentheses() {
        let caption = post("hatsune_miku_(vocaloid)").training_caption(ContentType::Sfw);
        assert_eq!(
            caption,
            "masterpiece, highest quality, sfw, hatsune miku \\(vocaloid\\)"
        );
    }

    #[test]
    fn extension_comes_from_the_last_dot() {
        assert_eq!(Post::ext_from_url("https://x.test/a/b.jpeg"), "jpeg");
    }

    #[test]
    fn answers_other_than_yes_are_safe() {
        assert_eq!(ContentType::from_answer("Y"), ContentType::Nsfw);
        assert_eq!(ContentType::from_answer("n"), ContentType::Sfw);
        assert_eq!(ContentType::from_answer(""), ContentType::Sfw);
    }
}
