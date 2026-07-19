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
