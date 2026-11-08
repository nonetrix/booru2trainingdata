# booru2trainingdata

This tool takes a Danbooru (or Gelbooru) post ID and downloads the image plus a
caption file, in a format best for training or fine-tuning anime txt2img models.

## Requirements

- Rust (edition 2024, so 1.85 or newer)

## Build

```sh
cargo build --release
```

The binary ends up at `target/release/booru2trainingdata`.

## Run

```sh
cargo run --release
```

The tool asks, in order:

1. **Directory** to save images to. If it does not exist you can have it created.
2. **Booru service**:
   1. **Danbooru** — what most anime models are already trained on, no credentials needed.
   2. **Gelbooru** — different tag conventions, less censorship without a Gold
      account. Needs an API key and user ID.
   3. **Gelbooru old** — the older unauthenticated API, the same one
      rule34.xxx runs. Needs no credentials.
   For options 2 and 3 you can point the tool at another instance (for example a
   `rule34.xxx` clone) instead of the default URL.
3. **NSFW or not** — the answer becomes a tag in every caption.

Afterwards it keeps asking for post IDs until the input ends (`Ctrl-D`, or
`Ctrl-C` to stop right away).

For each post it writes two files into the chosen directory:

- `<post id>.<ext>` — the full size image
- `<post id>.txt` — the caption:

  ```
  masterpiece, highest quality, <sfw|nsfw>, tag one, tag two, tags with \(parentheses\)
  ```

  Underscores become spaces, tags become comma separated, and parentheses are
  escaped for prompt syntax.

## Layout

- `src/main.rs` — the interactive prompts and the id loop
- `src/boorus.rs` — Danbooru and Gelbooru API clients
- `src/post.rs` — the post model and caption formatting
- `src/download.rs` — downloading the image and writing both files
- `src/prompt.rs` — stdin/stdout prompting helpers
- `tests/boorus.rs` — client and download tests against a local mock server

## Tests

```sh
cargo test
```

The tests use a loopback HTTP server, so they do not touch the real boorus.
