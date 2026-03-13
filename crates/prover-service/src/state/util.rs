pub(crate) fn sanitize_request_id_for_path(k: &str) -> String {
    // Percent-encode bytes outside a conservative safe set for filesystem friendliness.
    // We keep alnum and a small set of punctuation to retain readability.
    let mut out = String::with_capacity(k.len());
    for b in k.bytes() {
        let c = b as char;
        let safe = c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~');
        if safe {
            out.push(c);
        } else {
            out.push('%');
            out.push_str(&format!("{b:02X}"));
        }
    }
    out
}
