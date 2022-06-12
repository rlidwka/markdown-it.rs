use crate::MarkdownIt;
use crate::core::State;

pub fn add(md: &mut MarkdownIt) {
    md.core.ruler.push("inline", rule);
}

fn rule(state: &mut State) {
    // Parse inlines
    for token in &mut state.tokens {
        if token.name == "inline" {
            state.md.inline.parse(&token.content, state.md, &mut state.env, &mut token.children, 0);
        }
    }
}