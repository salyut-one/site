pub fn page(template: &str, title: &str, content: &str) -> String {
    template
        .replace("{{TITLE}}", &escape(title))
        .replace("{{CONTENT}}", content)
}

pub fn valid_username(username: &str) -> bool {
    if username.is_empty() || username.len() > 32 {
        return false;
    }
    let mut bytes = username.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first == b'_')
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
}

pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::{escape, valid_username};

    fn toplinks(template: &str) -> Vec<&str> {
        let (_, nav) = template
            .split_once("<nav class=\"toplinks\">")
            .expect("page has toplinks");
        let (nav, _) = nav.split_once("</nav>").expect("toplinks are closed");
        nav.lines()
            .map(str::trim)
            .filter(|line| line.starts_with("<a "))
            .collect()
    }

    #[test]
    fn managed_pages_have_consistent_toplinks() {
        let general = [
            r#"<a href="mailto:root@salyut.one">[Request Account]</a>"#,
            r#"<a href="mailto:root@salyut.one">[Support]</a>"#,
            r#"<a href="/bbs">[BBS]</a>"#,
            r#"<a href="/now">[Pinky]</a>"#,
        ];
        assert_eq!(toplinks(include_str!("../static/index.html")), general);
        assert_eq!(toplinks(include_str!("../templates/users.html")), general);
        assert_eq!(
            toplinks(include_str!("../templates/bbs.html")),
            [
                general[0],
                general[1],
                r#"<a href="/">[Home]</a>"#,
                general[3]
            ]
        );
        assert_eq!(
            toplinks(include_str!("../templates/now.html")),
            [
                general[0],
                general[1],
                general[2],
                r#"<a href="/">[Home]</a>"#
            ]
        );
    }

    #[test]
    fn validates_only_portable_unix_usernames() {
        assert!(valid_username("michal"));
        assert!(valid_username("_service"));
        assert!(valid_username("user-2"));
        for username in ["", "-root", "Bob", "a/b", "a.b", "a b"] {
            assert!(!valid_username(username), "{username:?} was accepted");
        }
    }

    #[test]
    fn escapes_html() {
        assert_eq!(
            escape("<script>alert('x') & \"y\"</script>"),
            "&lt;script&gt;alert(&#39;x&#39;) &amp; &quot;y&quot;&lt;/script&gt;"
        );
    }
}
