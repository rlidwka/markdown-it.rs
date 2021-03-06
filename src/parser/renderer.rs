use std::fmt::Debug;
use crate::Node;
use crate::common::utils::escape_html;

/// Each node outputs its HTML using this API.
///
/// Renderer is a struct that walks through AST and collects HTML from each node
/// into internal buffer.
pub trait Renderer {
    /// Write opening html tag with attributes, e.g. `<a href="url">`.
    fn open(&mut self, tag: &str, attrs: &[(&str, String)]);
    /// Write closing html tag, e.g. `</a>`.
    fn close(&mut self, tag: &str);
    /// Write self-closing html tag with attributes, e.g. `<img src="url"/>`.
    fn self_close(&mut self, tag: &str, attrs: &[(&str, String)]);
    /// Loop through child nodes and render each one.
    fn contents(&mut self, nodes: &[Node]);
    /// Write line break (`\n`). Default renderer ignores it if last char in the buffer is `\n` already.
    fn cr(&mut self);
    /// Write plain text with escaping, `<div>` -> `&lt;div&gt;`.
    fn text(&mut self, text: &str);
    /// Write plain text without escaping, `<div>` -> `<div>`.
    fn text_raw(&mut self, text: &str);
}

#[derive(Debug, Default)]
/// Default HTML/XHTML renderer.
pub(crate) struct HTMLRenderer<const XHTML: bool> {
    result: String,
}

impl<const XHTML: bool> HTMLRenderer<XHTML> {
    pub fn new() -> Self {
        Self {
            result: String::new(),
        }
    }

    pub fn render(&mut self, node: &Node) {
        node.node_value.render(node, self);
    }

    fn make_attr(&mut self, name: &str, value: &str) {
        self.result.push(' ');
        self.result.push_str(&escape_html(name));
        self.result.push('=');
        self.result.push('"');
        self.result.push_str(&escape_html(value));
        self.result.push('"');
    }

    fn make_attrs(&mut self, attrs: &[(&str, String)]) {
        for (name, value) in attrs {
            self.make_attr(name, value);
        }
    }
}

impl<const XHTML: bool> From<HTMLRenderer<XHTML>> for String {
    fn from(f: HTMLRenderer<XHTML>) -> Self {
        #[cold]
        fn replace_null(input: String) -> String {
            input.replace('\0', "\u{FFFD}")
        }

        if f.result.contains('\0') {
            // U+0000 must be replaced with U+FFFD as per commonmark spec,
            // we do it at the very end in order to avoid messing with byte offsets
            // for source maps (since "\0".len() != "\u{FFFD}".len())
            replace_null(f.result)
        } else {
            f.result
        }
    }
}

impl<const XHTML: bool> Renderer for HTMLRenderer<XHTML> {
    fn open(&mut self, tag: &str, attrs: &[(&str, String)]) {
        self.result.push('<');
        self.result.push_str(tag);
        self.make_attrs(attrs);
        self.result.push('>');
    }

    fn close(&mut self, tag: &str) {
        self.result.push('<');
        self.result.push('/');
        self.result.push_str(tag);
        self.result.push('>');
    }

    fn self_close(&mut self, tag: &str, attrs: &[(&str, String)]) {
        self.result.push('<');
        self.result.push_str(tag);
        self.make_attrs(attrs);
        if XHTML {
            self.result.push(' ');
            self.result.push('/');
        }
        self.result.push('>');
    }

    fn contents(&mut self, nodes: &[Node]) {
        for node in nodes.iter() {
            self.render(node);
        }
    }

    fn cr(&mut self) {
        // only push '\n' if last character isn't it
        match self.result.as_bytes().last() {
            Some(b'\n') | None => {}
            Some(_) => self.result.push('\n')
        }
    }

    fn text(&mut self, text: &str) {
        self.result.push_str(&escape_html(text));
    }

    fn text_raw(&mut self, text: &str) {
        self.result.push_str(text);
    }
}
