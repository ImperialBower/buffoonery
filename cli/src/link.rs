//! A board as a link to the page: the same fragment the page's `boardHash`
//! writes (`web/src/model.js`), so `readHash` reads it back.

/// The page on GitHub Pages.
pub const PAGE: &str = "https://imperialbower.github.io/buffoonery/";

/// The page URL that opens this board: `{base}#seed=…&effects=…&board=…`,
/// each part before the board only when set. `spec_json` is the board as
/// JSON (`parse`'s output); `effects_json` is the list the page keeps,
/// `[{"id":1,"op":{…}}]`.
#[must_use]
pub fn board_link(
    base: &str,
    spec_json: &str,
    seed: Option<u64>,
    effects_json: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    if let Some(seed) = seed {
        parts.push(format!("seed={seed}"));
    }
    if let Some(effects) = effects_json {
        parts.push(format!("effects={}", encode_uri_component(effects)));
    }
    parts.push(format!("board={}", encode_uri_component(spec_json)));
    format!("{base}#{}", parts.join("&"))
}

/// Percent-encodes `text` as JavaScript's `encodeURIComponent` does: every
/// byte of its UTF-8 form except `A-Z a-z 0-9 - _ . ! ~ * ' ( )`.
#[must_use]
pub fn encode_uri_component(text: &str) -> String {
    use std::fmt::Write;
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            // Writing to a `String` cannot fail.
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_link_board_only() {
        assert_eq!(
            board_link("http://x/", r#"{"played":["AS"]}"#, None, None),
            "http://x/#board=%7B%22played%22%3A%5B%22AS%22%5D%7D"
        );
    }

    #[test]
    fn board_link_with_seed_and_effects() {
        assert_eq!(
            board_link(
                "http://x/",
                r#"{"played":["AS"]}"#,
                Some(7),
                Some(r#"[{"id":1,"op":{"TimesMult":2.0}}]"#)
            ),
            "http://x/#seed=7\
             &effects=%5B%7B%22id%22%3A1%2C%22op%22%3A%7B%22TimesMult%22%3A2.0%7D%7D%5D\
             &board=%7B%22played%22%3A%5B%22AS%22%5D%7D"
        );
    }

    #[test]
    fn board_link_encodes_like_encode_uri_component() {
        // From Node: encodeURIComponent("a b+c/é♠-_.!~*'()&=#%")
        assert_eq!(
            encode_uri_component("a b+c/é♠-_.!~*'()&=#%"),
            "a%20b%2Bc%2F%C3%A9%E2%99%A0-_.!~*'()%26%3D%23%25"
        );
    }
}
