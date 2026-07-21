use chrono::{DateTime, SecondsFormat, Utc};
use salyut_bbs::protocol::{Board, Post};

const BASE_URL: &str = "https://salyut.one";
const ENTRY_LIMIT: usize = 50;
pub(super) const THREAD_LIMIT: u32 = 50;

#[derive(Clone, Copy)]
pub(super) enum Format {
    Rss,
    Atom,
}

impl Format {
    pub(super) const fn content_type(self) -> &'static str {
        match self {
            Self::Rss => "application/rss+xml; charset=utf-8",
            Self::Atom => "application/atom+xml; charset=utf-8",
        }
    }
}

struct Entry<'a> {
    id: String,
    title: String,
    author: &'a str,
    body: &'a str,
    published: DateTime<Utc>,
    updated: DateTime<Utc>,
}

pub(super) fn render(format: Format, board: &Board, posts: &[Post]) -> String {
    let mut entries = entries(posts);
    entries.sort_by(|left, right| {
        right
            .updated
            .cmp(&left.updated)
            .then_with(|| right.id.cmp(&left.id))
    });
    entries.truncate(ENTRY_LIMIT);
    match format {
        Format::Rss => rss(board, &entries),
        Format::Atom => atom(board, &entries),
    }
}

fn entries(posts: &[Post]) -> Vec<Entry<'_>> {
    posts
        .iter()
        .flat_map(|post| {
            let post_url = format!("{BASE_URL}/bbs/posts/{}", post.id);
            let root = Entry {
                id: post_url.clone(),
                title: post.title.clone(),
                author: &post.author,
                body: &post.body,
                published: post.created_at,
                updated: post.created_at,
            };
            std::iter::once(root).chain(post.replies.iter().map(move |reply| Entry {
                id: format!("{post_url}#reply-{}", reply.id),
                title: format!("Re: {}", post.title),
                author: &reply.author,
                body: &reply.body,
                published: reply.created_at,
                updated: reply.updated_at,
            }))
        })
        .collect()
}

fn rss(board: &Board, entries: &[Entry<'_>]) -> String {
    let board_url = board_url(board);
    let feed_url = format!("{board_url}/rss.xml");
    let title = format!("salyut.one BBS — {}", board.name);
    let last_build = entries
        .first()
        .map_or_else(epoch, |entry| entry.updated)
        .to_rfc2822();
    let items = entries
        .iter()
        .map(|entry| {
            format!(
                "<item><title>{}</title><link>{}</link>\
                 <guid isPermaLink=\"true\">{}</guid><dc:creator>{}</dc:creator>\
                 <pubDate>{}</pubDate><description>{}</description></item>",
                xml(&entry.title),
                xml(&entry.id),
                xml(&entry.id),
                xml(entry.author),
                entry.published.to_rfc2822(),
                xml(entry.body),
            )
        })
        .collect::<String>();
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\
         <rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\" \
         xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><channel>\
         <title>{}</title><link>{}</link><description>{}</description>\
         <lastBuildDate>{last_build}</lastBuildDate>\
         <atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\"/>\
         {items}</channel></rss>\n",
        xml(&title),
        xml(&board_url),
        xml(&board.description),
        xml(&feed_url),
    )
}

fn atom(board: &Board, entries: &[Entry<'_>]) -> String {
    let board_url = board_url(board);
    let feed_url = format!("{board_url}/atom.xml");
    let title = format!("salyut.one BBS — {}", board.name);
    let updated = entries.first().map_or_else(epoch, |entry| entry.updated);
    let items = entries
        .iter()
        .map(|entry| {
            format!(
                "<entry><title>{}</title><id>{}</id><link href=\"{}\"/>\
                 <published>{}</published><updated>{}</updated>\
                 <author><name>{}</name></author><content type=\"text\">{}</content></entry>",
                xml(&entry.title),
                xml(&entry.id),
                xml(&entry.id),
                atom_date(entry.published),
                atom_date(entry.updated),
                xml(entry.author),
                xml(entry.body),
            )
        })
        .collect::<String>();
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\
         <feed xmlns=\"http://www.w3.org/2005/Atom\"><title>{}</title>\
         <subtitle>{}</subtitle><id>{}</id><updated>{}</updated>\
         <link href=\"{}\"/><link href=\"{}\" rel=\"self\" \
         type=\"application/atom+xml\"/>{items}</feed>\n",
        xml(&title),
        xml(&board.description),
        xml(&board_url),
        atom_date(updated),
        xml(&board_url),
        xml(&feed_url),
    )
}

fn board_url(board: &Board) -> String {
    format!("{BASE_URL}/bbs/boards/{}", board.slug)
}

fn atom_date(date: DateTime<Utc>) -> String {
    date.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn epoch() -> DateTime<Utc> {
    DateTime::UNIX_EPOCH
}

fn xml(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            matches!(*character, '\u{9}' | '\u{a}' | '\u{d}')
                || matches!(*character as u32, 0x20..=0xd7ff | 0xe000..=0xfffd | 0x10000..=0x10ffff)
        })
        .fold(String::new(), |mut escaped, character| {
            match character {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                '"' => escaped.push_str("&quot;"),
                '\'' => escaped.push_str("&apos;"),
                _ => escaped.push(character),
            }
            escaped
        })
}

#[cfg(test)]
mod tests {
    use super::{Format, render};

    fn fixture() -> (salyut_bbs::protocol::Board, Vec<salyut_bbs::protocol::Post>) {
        serde_json::from_value(serde_json::json!({
            "board": {
                "id": 1,
                "slug": "general",
                "name": "General & chat",
                "description": "General <discussion>",
                "kind": "discussion",
                "write_group": null
            },
            "posts": [{
                "id": 7,
                "board": {
                    "id": 1,
                    "slug": "general",
                    "name": "General & chat",
                    "description": "General <discussion>",
                    "kind": "discussion",
                    "write_group": null
                },
                "author": "alice & bob",
                "title": "Hello <world>",
                "body": "Root & body\u{0001}",
                "locked": false,
                "replies": [{
                    "id": 9,
                    "author": "carol",
                    "body": "A <reply>",
                    "created_at": "2026-07-21T12:00:00Z",
                    "updated_at": "2026-07-21T12:01:00Z"
                }],
                "poll": null,
                "proposal": null,
                "created_at": "2026-07-20T10:00:00Z",
                "updated_at": "2026-07-20T10:00:00Z"
            }]
        }))
        .map(|fixture: serde_json::Value| {
            (
                serde_json::from_value(fixture["board"].clone()).unwrap(),
                serde_json::from_value(fixture["posts"].clone()).unwrap(),
            )
        })
        .unwrap()
    }

    #[test]
    fn rss_contains_stable_thread_and_reply_items() {
        let (board, posts) = fixture();
        let feed = render(Format::Rss, &board, &posts);
        assert!(feed.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
        assert!(feed.contains("<title>salyut.one BBS — General &amp; chat</title>"));
        assert!(feed.contains("https://salyut.one/bbs/posts/7#reply-9"));
        assert!(feed.contains("<title>Re: Hello &lt;world&gt;</title>"));
        assert!(feed.contains("<description>A &lt;reply&gt;</description>"));
        assert!(!feed.contains('\u{1}'));
    }

    #[test]
    fn atom_uses_reply_time_as_feed_update() {
        let (board, posts) = fixture();
        let feed = render(Format::Atom, &board, &posts);
        assert!(feed.contains("<updated>2026-07-21T12:01:00Z</updated>"));
        assert!(feed.contains("<author><name>alice &amp; bob</name></author>"));
        assert!(feed.contains("<content type=\"text\">Root &amp; body</content>"));
        assert!(feed.contains(
            "<link href=\"https://salyut.one/bbs/boards/general/atom.xml\" rel=\"self\""
        ));
    }
}
