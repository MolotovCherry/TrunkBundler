use minify_html::{minify, Cfg};

pub fn process_html(content: &str) -> String {
    let mut cfg = Cfg::spec_compliant();
    cfg.minify_css = true;
    cfg.minify_js = true;
    cfg.keep_closing_tags = true;

    let minified = minify(content.as_bytes(), &cfg);

    String::from_utf8_lossy(&minified).into_owned()
}
