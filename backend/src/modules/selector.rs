use crate::modules::domnode::DomNode;
use crate::modules::tree::{LcaBinaryLifting, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    Child,
    Descendant,
    AdjacentSibling,
    GeneralSibling,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSelector {
    Universal,
    Tag(String),
    Class(String),
    Id(String),
    AttrEq { key: String, value: String },
}

#[derive(Debug, Clone)]
pub struct SelectorPart {
    pub selector: SimpleSelector,
    pub relation_to_left: Option<Combinator>,
}

#[derive(Debug, Clone)]
pub struct SelectorQuery {
    pub parts_right_to_left: Vec<SelectorPart>,
}

impl SelectorQuery {
    pub fn parse(input: &str) -> Result<Self, String> {
        parse_selector(input)
    }
}

pub fn parse_selector(input: &str) -> Result<SelectorQuery, String> {
    let mut selectors_left_to_right: Vec<String> = Vec::new();
    let mut combinators_left_to_right: Vec<Combinator> = Vec::new();
    let mut buffer = String::new();
    let mut bracket_depth = 0usize;
    let mut pending_descendant = false;
    let mut last_was_selector = false;

    let flush_selector = |buffer: &mut String,
                          pending_descendant: &mut bool,
                          last_was_selector: &mut bool,
                          selectors: &mut Vec<String>,
                          combinators: &mut Vec<Combinator>| {
        let token = buffer.trim();
        if token.is_empty() {
            buffer.clear();
            return;
        }

        if *pending_descendant && *last_was_selector {
            combinators.push(Combinator::Descendant);
        }

        selectors.push(token.to_string());
        *last_was_selector = true;
        *pending_descendant = false;
        buffer.clear();
    };

    for ch in input.chars() {
        match ch {
            '[' => {
                bracket_depth += 1;
                buffer.push(ch);
            }
            ']' => {
                if bracket_depth == 0 {
                    return Err("Selector bracket mismatch".to_string());
                }
                bracket_depth -= 1;
                buffer.push(ch);
            }
            '>' | '+' | '~' if bracket_depth == 0 => {
                flush_selector(
                    &mut buffer,
                    &mut pending_descendant,
                    &mut last_was_selector,
                    &mut selectors_left_to_right,
                    &mut combinators_left_to_right,
                );

                if !last_was_selector {
                    return Err("Invalid combinator placement".to_string());
                }

                let combinator = match ch {
                    '>' => Combinator::Child,
                    '+' => Combinator::AdjacentSibling,
                    '~' => Combinator::GeneralSibling,
                    _ => unreachable!(),
                };
                combinators_left_to_right.push(combinator);
                last_was_selector = false;
                pending_descendant = false;
            }
            c if c.is_whitespace() && bracket_depth == 0 => {
                flush_selector(
                    &mut buffer,
                    &mut pending_descendant,
                    &mut last_was_selector,
                    &mut selectors_left_to_right,
                    &mut combinators_left_to_right,
                );

                if last_was_selector {
                    pending_descendant = true;
                }
            }
            _ => buffer.push(ch),
        }
    }

    flush_selector(
        &mut buffer,
        &mut pending_descendant,
        &mut last_was_selector,
        &mut selectors_left_to_right,
        &mut combinators_left_to_right,
    );

    if bracket_depth != 0 {
        return Err("Selector bracket mismatch".to_string());
    }

    if selectors_left_to_right.is_empty() {
        return Err("Empty selector".to_string());
    }

    if combinators_left_to_right.len() + 1 != selectors_left_to_right.len() {
        return Err("Invalid selector chain".to_string());
    }

    let mut parts_right_to_left = Vec::new();
    for selector_idx in (0..selectors_left_to_right.len()).rev() {
        let selector = parse_simple_selector(&selectors_left_to_right[selector_idx])?;
        let relation_to_left = if selector_idx > 0 {
            Some(combinators_left_to_right[selector_idx - 1])
        } else {
            None
        };

        parts_right_to_left.push(SelectorPart {
            selector,
            relation_to_left,
        });
    }

    Ok(SelectorQuery {
        parts_right_to_left,
    })
}

pub fn query_matches_node(
    tree: &Tree,
    lca: &LcaBinaryLifting,
    node_id: usize,
    query: &SelectorQuery,
) -> bool {
    if query.parts_right_to_left.is_empty() {
        return false;
    }

    let mut current_id = node_id;

    for part_idx in 0..query.parts_right_to_left.len() {
        let part = &query.parts_right_to_left[part_idx];
        let current_node = match get_node(tree, current_id) {
            Some(node) => node,
            None => return false,
        };

        if !matches_simple_selector(current_node, &part.selector) {
            return false;
        }

        let relation = match part.relation_to_left {
            Some(rel) => rel,
            None => continue,
        };

        let left_part = match query.parts_right_to_left.get(part_idx + 1) {
            Some(p) => p,
            None => return false,
        };

        let next_id = match match_left_node(tree, lca, current_id, relation, &left_part.selector) {
            Some(id) => id,
            None => return false,
        };

        current_id = next_id;
    }

    true
}

pub fn find_matching_nodes(tree: &Tree, query: &SelectorQuery) -> Vec<usize> {
    let lca = tree.build_lca_index();
    tree.nodes
        .iter()
        .map(|node| node.id())
        .filter(|node_id| query_matches_node(tree, &lca, *node_id, query))
        .collect()
}

fn parse_simple_selector(token: &str) -> Result<SimpleSelector, String> {
    if token == "*" {
        return Ok(SimpleSelector::Universal);
    }

    if let Some(rest) = token.strip_prefix('.') {
        if rest.is_empty() {
            return Err("Class selector cannot be empty".to_string());
        }
        return Ok(SimpleSelector::Class(rest.to_string()));
    }

    if let Some(rest) = token.strip_prefix('#') {
        if rest.is_empty() {
            return Err("ID selector cannot be empty".to_string());
        }
        return Ok(SimpleSelector::Id(rest.to_string()));
    }

    if token.starts_with('[') && token.ends_with(']') {
        let content = &token[1..token.len() - 1];
        let mut parts = content.splitn(2, '=');
        let key = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim();

        if key.is_empty() || value.is_empty() {
            return Err("Attribute selector must be in [key=value] format".to_string());
        }

        let normalized_value = value.trim_matches('"').trim_matches('\'');
        return Ok(SimpleSelector::AttrEq {
            key: key.to_string(),
            value: normalized_value.to_string(),
        });
    }

    Ok(SimpleSelector::Tag(token.to_string()))
}

fn get_node(tree: &Tree, node_id: usize) -> Option<&DomNode> {
    tree.find_index_from_id(node_id)
        .and_then(|index| tree.nodes.get(index))
}

fn matches_simple_selector(node: &DomNode, selector: &SimpleSelector) -> bool {
    match selector {
        SimpleSelector::Universal => true,
        SimpleSelector::Tag(tag) => node.tag().eq_ignore_ascii_case(tag),
        SimpleSelector::Class(class_name) => node
            .attrs()
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("class"))
            .map(|(_, value)| {
                value
                    .split_whitespace()
                    .any(|part| part.eq_ignore_ascii_case(class_name))
            })
            .unwrap_or(false),
        SimpleSelector::Id(id_value) => node
            .attrs()
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("id"))
            .map(|(_, value)| value == id_value)
            .unwrap_or(false),
        SimpleSelector::AttrEq { key, value } => node.attrs().iter().any(|(k, v)| {
            k.eq_ignore_ascii_case(key) && v == value
        }),
    }
}

fn match_left_node(
    tree: &Tree,
    lca: &LcaBinaryLifting,
    right_id: usize,
    relation: Combinator,
    left_selector: &SimpleSelector,
) -> Option<usize> {
    match relation {
        Combinator::Child => {
            let right_node = get_node(tree, right_id)?;
            let parent_id = right_node.parent()?;
            let parent_node = get_node(tree, parent_id)?;
            if matches_simple_selector(parent_node, left_selector) {
                Some(parent_id)
            } else {
                None
            }
        }
        Combinator::Descendant => {
            let mut current = get_node(tree, right_id)?.parent();
            while let Some(ancestor_id) = current {
                if !lca.is_ancestor_id(tree, ancestor_id, right_id) {
                    current = get_node(tree, ancestor_id).and_then(|n| n.parent());
                    continue;
                }

                let ancestor_node = get_node(tree, ancestor_id)?;
                if matches_simple_selector(ancestor_node, left_selector) {
                    return Some(ancestor_id);
                }

                current = ancestor_node.parent();
            }
            None
        }
        Combinator::AdjacentSibling => {
            let right_node = get_node(tree, right_id)?;
            let parent_id = right_node.parent()?;
            let parent_node = get_node(tree, parent_id)?;
            let siblings = parent_node.children();
            let right_pos = siblings.iter().position(|id| *id == right_id)?;

            if right_pos == 0 {
                return None;
            }

            let candidate_id = siblings[right_pos - 1];
            let candidate_node = get_node(tree, candidate_id)?;
            if matches_simple_selector(candidate_node, left_selector) {
                Some(candidate_id)
            } else {
                None
            }
        }
        Combinator::GeneralSibling => {
            let right_node = get_node(tree, right_id)?;
            let parent_id = right_node.parent()?;
            let parent_node = get_node(tree, parent_id)?;
            let siblings = parent_node.children();
            let right_pos = siblings.iter().position(|id| *id == right_id)?;

            for candidate_id in siblings[..right_pos].iter().rev() {
                let candidate_node = get_node(tree, *candidate_id)?;
                if matches_simple_selector(candidate_node, left_selector) {
                    return Some(*candidate_id);
                }
            }

            None
        }
    }
}
