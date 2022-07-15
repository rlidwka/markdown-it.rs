// Parse backticks
//
use crate::Node;
use crate::parser::MarkdownIt;
use crate::parser::internals::inline;
use crate::parser::internals::syntax_base::builtin::Text;

#[derive(Debug, Default)]
struct BacktickCache<const MARKER: char> {
    scanned: bool,
    max: Vec<usize>,
}

#[derive(Debug)]
struct BacktickCfg<const MARKER: char>(fn (usize) -> Node);

pub fn add_with<const MARKER: char>(md: &mut MarkdownIt, f: fn (usize) -> Node) {
    md.env.insert(BacktickCfg::<MARKER>(f));

    md.inline.ruler.add("generic::code_pair", rule::<MARKER>);
}

fn rule<const MARKER: char>(state: &mut inline::State, silent: bool) -> bool {
    let mut chars = state.src[state.pos..state.pos_max].chars();
    if chars.next().unwrap() != MARKER { return false; }
    if state.trailing_text_get().ends_with(MARKER) { return false; }

    let mut pos = state.pos + 1;

    // scan marker length
    while Some(MARKER) == chars.next() {
        pos += 1;
    }

    // backtick length => last seen position
    let backticks = state.inline_env.get_or_insert_default::<BacktickCache<MARKER>>();
    let opener_len = pos - state.pos;

    if backticks.scanned && backticks.max[opener_len] <= state.pos {
        // performance note: adding entire sequence into pending is 5x faster,
        // but it will interfere with other rules working on the same char;
        // and it is extremely rare that user would put a thousand "`" in text
        return false;
    }

    let mut match_start;
    let mut match_end = pos;

    // Nothing found in the cache, scan until the end of the line (or until marker is found)
    loop {
        match state.src[match_end..state.pos_max].find(MARKER) {
            Some(x) => { match_start = match_end + x; }
            None =>    { break; }
        }

        // scan marker length
        match_end = match_start + 1;
        chars = state.src[match_end..state.pos_max].chars();

        while Some(MARKER) == chars.next() {
            match_end += 1;
        }

        let closer_len = match_end - match_start;

        if closer_len == opener_len {
            // Found matching closer length.
            if !silent {
                let mut content = state.src[pos..match_start].to_owned().replace('\n', " ");
                if content.starts_with(' ') && content.ends_with(' ') && content.len() > 2 {
                    content = content[1..content.len() - 1].to_owned();
                    pos += 1;
                    match_start -= 1;
                }

                let f = state.md.env.get::<BacktickCfg<MARKER>>().unwrap().0;
                let mut node = f(opener_len);
                node.srcmap = state.get_map(state.pos, match_end);

                let mut inner_node = Node::new(Text { content });
                inner_node.srcmap = state.get_map(pos, match_start);

                node.children.push(inner_node);
                state.push(node);
            }
            state.pos = match_end;
            return true;
        }

        // Some different length found, put it in cache as upper limit of where closer can be found
        while backticks.max.len() <= closer_len { backticks.max.push(0); }
        backticks.max[closer_len] = match_start;
    }

    // Scanned through the end, didn't find anything
    backticks.scanned = true;

    false
}
