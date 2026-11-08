//! Client and download tests against a tiny loopback HTTP server, so none of
//! them need the real boorus to be reachable.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use booru2trainingdata::boorus::{Booru, Credentials, Danbooru, Gelbooru, Http};
use booru2trainingdata::download;
use booru2trainingdata::post::{ContentType, Post};

/// Serve `responses` (status, content type, body) in order on a loopback port.
///
/// Returns the base URL and a channel yielding each request line the server saw.
fn serve(responses: Vec<(u16, &'static str, Vec<u8>)>) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback port");
    let base = format!("http://{}", listener.local_addr().expect("local addr"));
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        for (status, content_type, body) in responses {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));

            let mut request_line = String::new();
            reader.read_line(&mut request_line).expect("request line");
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).expect("header") == 0 || header == "\r\n" {
                    break;
                }
            }
            let _ = sender.send(request_line.trim_end().to_owned());

            write!(
                stream,
                "HTTP/1.1 {status} Test\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .expect("write head");
            stream.write_all(&body).expect("write body");
            stream.flush().expect("flush");
        }
    });

    (base, receiver)
}

fn http() -> Http {
    Http::new().expect("http client")
}

#[test]
fn danbooru_post_is_parsed() {
    let body = br#"{"id":123,"file_url":"https://img.test/abc.png","file_ext":"png","tag_string":"1girl solo"}"#;
    let (base, requests) = serve(vec![(200, "application/json", body.to_vec())]);

    let post = Danbooru::new(&base, &http())
        .expect("client")
        .fetch("123")
        .expect("request")
        .expect("post");

    assert_eq!(
        post,
        Post {
            id: "123".to_owned(),
            file_url: "https://img.test/abc.png".to_owned(),
            file_ext: "png".to_owned(),
            tags: "1girl solo".to_owned(),
        }
    );
    assert_eq!(requests.recv().unwrap(), "GET /posts/123.json HTTP/1.1");
}

#[test]
fn danbooru_post_without_file_url_is_skipped() {
    let body = br#"{"id":9,"file_url":null,"file_ext":"png","tag_string":"1girl"}"#;
    let (base, _requests) = serve(vec![(200, "application/json", body.to_vec())]);

    let post = Danbooru::new(&base, &http())
        .expect("client")
        .fetch("9")
        .expect("request");

    assert_eq!(post, None);
}

#[test]
fn gelbooru_sends_credentials_and_parses_wrapped_posts() {
    let body = br#"{"@attributes":{"limit":1},"post":[{"id":42,"file_url":"https://img.test/42.jpeg","tags":"1girl solo"}]}"#;
    let (base, requests) = serve(vec![(200, "application/json", body.to_vec())]);

    let credentials = Credentials {
        api_key: "secret".to_owned(),
        user_id: "7".to_owned(),
    };
    let post = Gelbooru::new(&base, Some(credentials), &http())
        .expect("client")
        .fetch("42")
        .expect("request")
        .expect("post");

    assert_eq!(post.id, "42");
    assert_eq!(post.file_ext, "jpeg");
    assert_eq!(post.tags, "1girl solo");

    let request = requests.recv().unwrap();
    assert!(request.starts_with("GET /index.php?"), "{request}");
    for expected in ["id=42", "json=1", "api_key=secret", "user_id=7"] {
        assert!(
            request.contains(expected),
            "{expected} missing in {request}"
        );
    }
}

#[test]
fn legacy_gelbooru_takes_bare_arrays_and_sends_no_credentials() {
    let body = br#"[{"id":"42","file_url":"https://img.test/42.png","tags":"1girl"}]"#;
    let (base, requests) = serve(vec![(200, "application/json", body.to_vec())]);

    let post = Gelbooru::new(&base, None, &http())
        .expect("client")
        .fetch("42")
        .expect("request")
        .expect("post");

    assert_eq!(post.id, "42");
    assert_eq!(post.file_ext, "png");

    let request = requests.recv().unwrap();
    assert!(!request.contains("api_key"), "{request}");
    assert!(!request.contains("user_id"), "{request}");
}

#[test]
fn gelbooru_tags_are_html_decoded() {
    let body = br#"{"post":[{"id":7,"file_url":"https://img.test/7.png","tags":"1girl negev_(girls&#039;_frontline) hatsune_miku_&amp;_co"}]}"#;
    let (base, _requests) = serve(vec![(200, "application/json", body.to_vec())]);

    let post = Gelbooru::new(&base, None, &http())
        .expect("client")
        .fetch("7")
        .expect("request")
        .expect("post");

    assert_eq!(
        post.tags,
        "1girl negev_(girls'_frontline) hatsune_miku_&_co"
    );
    assert_eq!(
        post.training_caption(ContentType::Sfw),
        "masterpiece, highest quality, sfw, 1girl, negev \\(girls' frontline\\), hatsune miku & co"
    );
}

#[test]
fn empty_gelbooru_reply_is_not_a_post() {
    let body = br#"{"@attributes":{"limit":0}}"#;
    let (base, _requests) = serve(vec![(200, "application/json", body.to_vec())]);

    let post = Gelbooru::new(&base, None, &http())
        .expect("client")
        .fetch("404")
        .expect("request");

    assert_eq!(post, None);
}

#[test]
fn rejected_request_is_not_a_post() {
    let (base, _requests) = serve(vec![(404, "text/html", b"nope".to_vec())]);

    let post = Danbooru::new(&base, &http())
        .expect("client")
        .fetch("404")
        .expect("request");

    assert_eq!(post, None);
}

#[test]
fn save_writes_image_and_caption() {
    let image = vec![0x89, b'P', b'N', b'G'];
    let (base, _requests) = serve(vec![(200, "image/png", image.clone())]);

    let dir = std::env::temp_dir().join(format!("booru2trainingdata-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("temp dir");
    let post = Post {
        id: "7".to_owned(),
        file_url: format!("{base}/7.png"),
        file_ext: "png".to_owned(),
        tags: "hatsune_miku_(vocaloid) solo".to_owned(),
    };

    download::save(&post, &dir, ContentType::Nsfw, &http()).expect("save");

    assert_eq!(fs::read(dir.join("7.png")).expect("image"), image);
    assert_eq!(
        fs::read_to_string(dir.join("7.txt")).expect("caption"),
        "masterpiece, highest quality, nsfw, hatsune miku \\(vocaloid\\), solo"
    );

    fs::remove_dir_all(&dir).expect("clean up");
}
