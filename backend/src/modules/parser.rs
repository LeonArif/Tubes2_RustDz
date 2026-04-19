use crate::modules::tree::Tree;

const VOID_TAGS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

// fungsi parsing HTML menjadi Tree
pub fn parse_html_to_tree(html: &str) -> Result<Tree, String> {
    // menggunakan root sintetis untuk memastikan bahwa semua node memiliki parent,
    // sehingga traversal dan pencocokan selector dapat dilakukan dengan lebih konsisten,
    // bahkan untuk fragmen HTML yang tidak lengkap atau tidak valid
    let mut tree = Tree::new();
    tree.add_root("document".to_string(), Vec::new());

    let mut stack: Vec<usize> = vec![tree.root];
    let mut next_id = 1usize;
    let mut cursor = 0usize;

    while let Some(start_rel) = html[cursor..].find('<') {
        let start = cursor + start_rel;

        if html[start..].starts_with("<!--") {
            // skip comment untuk toleransi markup yang tidak sempurna
            // memastikan bahwa komentar yang dimulai dengan <!-- diakhiri dengan -->,
            // jika tidak ditemukan --> maka akan menganggap sisa string sebagai komentar dan berhenti parsing
            if let Some(end_rel) = html[start + 4..].find("-->") {
                cursor = start + 4 + end_rel + 3;
                continue;
            }
            break;
        }

        let end = match html[start..].find('>') {
            Some(rel) => start + rel,
            None => break,
        };

        let raw_token = html[start + 1..end].trim();
        if raw_token.is_empty() {
            cursor = end + 1;
            continue;
        }

        if raw_token.starts_with('!') || raw_token.starts_with('?') {
            cursor = end + 1;
            continue;
        }

        if let Some(rest) = raw_token.strip_prefix('/') {
            let closing_tag = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();

            if !closing_tag.is_empty() {
                // pop stack sampai menemukan tag pembuka yang sesuai dengan tag penutup,
                // untuk toleransi markup yang tidak sempurna
                while stack.len() > 1 {
                    let current_id = *stack.last().unwrap_or(&tree.root);
                    let current_index = match tree.find_index_from_id(current_id) {
                        Some(index) => index,
                        None => {
                            stack.pop();
                            continue;
                        }
                    };

                    let current_tag = tree.nodes[current_index].tag().to_ascii_lowercase();
                    stack.pop();

                    if current_tag == closing_tag {
                        break;
                    }
                }
            }

            cursor = end + 1;
            continue;
        }

        let self_closing = raw_token.ends_with('/');
        let normalized = if self_closing {
            raw_token[..raw_token.len() - 1].trim()
        } else {
            raw_token
        };

        let (tag, attrs) = parse_opening_tag(normalized);
        if tag.is_empty() {
            cursor = end + 1;
            continue;
        }

        let parent_id = *stack.last().unwrap_or(&tree.root);
        tree.add_child(next_id, tag.clone(), parent_id, attrs);

        // void/self-closing tags tidak dimasukkan ke stack karena tidak memiliki children
        if !self_closing && !is_void_tag(&tag) {
            stack.push(next_id);
        }

        next_id += 1;
        cursor = end + 1;
    }

    Ok(tree)
}

// fungsi parsing opening tag untuk mengekstrak nama tag dan atributnya
// fungsi ini menangani berbagai format atribut, termasuk yang menggunakan tanda kutip ganda, tunggal, atau tanpa tanda kutip
fn parse_opening_tag(input: &str) -> (String, Vec<(String, String)>) {
    let mut chars = input.chars().peekable();

    while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
        chars.next();
    }

    let mut tag = String::new();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_whitespace() {
            break;
        }
        tag.push(ch);
        chars.next();
    }

    let tag = tag.to_ascii_lowercase();
    let mut attrs = Vec::new();

    loop {
        while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
            chars.next();
        }

        if chars.peek().is_none() {
            break;
        }

        let mut key = String::new();
        while let Some(ch) = chars.peek().copied() {
            if ch.is_whitespace() || ch == '=' {
                break;
            }
            key.push(ch);
            chars.next();
        }

        if key.is_empty() {
            chars.next();
            continue;
        }

        while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
            chars.next();
        }

        let mut value = String::new();

        if matches!(chars.peek(), Some('=')) {
            chars.next();

            while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
                chars.next();
            }

            if let Some(quote @ ('"' | '\'')) = chars.peek().copied() {
                // handle tanda kutip ganda atau tunggal untuk nilai atribut,
                // memastikan bahwa nilai yang diambil adalah apa yang ada di dalam tanda kutip
                chars.next();

                while let Some(ch) = chars.peek().copied() {
                    chars.next();
                    if ch == quote {
                        break;
                    }
                    value.push(ch);
                }
            } else {
                // handle nilai atribut tanpa tanda kutip,
                // mengambil karakter sampai menemukan whitespace atau akhir string
                while let Some(ch) = chars.peek().copied() {
                    if ch.is_whitespace() {
                        break;
                    }
                    value.push(ch);
                    chars.next();
                }
            }
        }

        attrs.push((key.to_ascii_lowercase(), value));
    }

    (tag, attrs)
}

fn is_void_tag(tag: &str) -> bool {
    VOID_TAGS.iter().any(|t| t.eq_ignore_ascii_case(tag))
}
