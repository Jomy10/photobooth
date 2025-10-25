// use unic::emoji::char::is_emoji;
// use unic::segment::Graphemes;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy)]
pub enum TextSequenceKind {
    Text,
    Emoji
}

fn is_emoji(t: &str) -> bool {
    emojis::get(t).is_some()
}

pub fn split_emojis<'a>(text: &'a str) -> Vec<(TextSequenceKind, &'a str)> {
    let graphemes = UnicodeSegmentation::graphemes(text, true);
    let mut start = 0;
    let mut len = 0;

    let mut out = Vec::new();

    let mut iter = graphemes.into_iter().peekable();
    while let Some(grapheme) = iter.next() {
        if is_emoji(grapheme) {
            #[allow(unused_assignments)]
            if len > 0 {
                out.push((TextSequenceKind::Text, &text[start..(start + len)]));
                start = start + len;
                len = 0;
            }
            let mut emoji_len = grapheme.len();

            while let Some(c) = iter.peek() {
                if is_emoji(c) {
                    emoji_len += c.len();
                    _ = iter.next();
                } else {
                    break;
                }
            }

            out.push((TextSequenceKind::Emoji, &text[start..(start + emoji_len)]));
            start = start + emoji_len;
            len = 0;
        } else {
            len += grapheme.len();
        }
    }

    if len > 0 {
        out.push((TextSequenceKind::Text, &text[start..(start + len)]));
    }

    return out;
}
