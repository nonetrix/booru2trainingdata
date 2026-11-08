//! Turning what the user typed at a prompt into a real filesystem path.

use std::path::PathBuf;

/// Expand a leading `~` or `~/` to the current user's home directory.
///
/// A shell expands tilde before the program ever sees it, but input typed at a
/// prompt is used verbatim. Without this, `~/Downloads/pics` creates a directory
/// literally named `~` inside the working directory and the images vanish there.
///
/// Only a bare `~` and a `~/` (or `~\` on Windows) prefix are expanded; `~user`
/// forms are left alone because they are not portable.
pub fn expand_tilde(input: &str) -> PathBuf {
    let input = input.trim();
    let Some(rest) = input.strip_prefix('~') else {
        return PathBuf::from(input);
    };
    if !rest.is_empty() && !rest.starts_with(['/', '\\']) {
        return PathBuf::from(input);
    }

    match dirs::home_dir() {
        Some(home) => home.join(rest.trim_start_matches(['/', '\\'])),
        None => PathBuf::from(input),
    }
}

#[cfg(test)]
mod tests {
    use super::expand_tilde;
    use std::path::PathBuf;

    #[test]
    fn bare_tilde_is_the_home_directory() {
        let home = dirs::home_dir().expect("home directory");
        assert_eq!(expand_tilde("~"), home);
        assert_eq!(expand_tilde("~/"), home);
    }

    #[test]
    fn tilde_prefix_is_replaced_by_home() {
        let home = dirs::home_dir().expect("home directory");
        assert_eq!(
            expand_tilde("~/Downloads/pics"),
            home.join("Downloads").join("pics")
        );
    }

    #[test]
    fn other_paths_are_left_alone() {
        assert_eq!(
            expand_tilde("downloads/pics"),
            PathBuf::from("downloads/pics")
        );
        assert_eq!(expand_tilde("/tmp/pics"), PathBuf::from("/tmp/pics"));
        // Not a home directory form, so the shell would not expand it either.
        assert_eq!(expand_tilde("~other/pics"), PathBuf::from("~other/pics"));
        assert_eq!(expand_tilde("a/~/b"), PathBuf::from("a/~/b"));
    }
}
