use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::{ElementFlags, TreeSink};
use html5ever::{self, namespace_url, ns, Attribute, LocalName, QualName};
use markup5ever_rcdom::NodeData::{Element, Text, Document};
use markup5ever_rcdom::{Handle, Node, RcDom};
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::rc::Rc;
use std::str::FromStr;
use url::Url;

// @FIXME: remove bool, using lambda, internal function
pub fn query_by_tagname(handle: Handle, tag_name: &str, nodes: &mut Vec<Rc<Node>>) -> bool {
    for child in handle.children.borrow().iter() {
        match child.data {
            Element { ref name, .. } => {
                if name.local.as_ref().to_lowercase() == tag_name {
                    nodes.push(child.clone());
                }
                query_by_tagname(child.clone(), tag_name, nodes);
            }
            _ => {}
        }
    }

    nodes.len() > 0
}

pub fn get_text_content(handle: Handle, text: &mut String) {
    for child in handle.children.borrow().iter() {
        match child.data {
            Text { ref contents } => {
                text.push_str(contents.borrow().trim());
            }
            _ => {
                get_text_content(child.clone(), text);
            }
        }
    }

    if text.len() == 0 {
        text.push_str("Untitled");
    }
}

pub fn get_children_text_content(handle: Handle, text: &mut String, deep: bool) {
    for child in handle.children.borrow().iter() {
        let c = child.clone();
        match c.data {
            Text { ref contents } => {
                text.push_str(contents.borrow().trim());
            }
            Element { .. } => {
                if deep {
                    get_children_text_content(child.clone(), text, deep);
                }
            }
            _ => (),
        }
    }
}
pub fn parse_from<R>(input: &mut R) -> Result<RcDom, Box<dyn Error>>
where
    R: Read,
{
    Ok(
        html5ever::parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(input)
            .unwrap(),
    )
}

pub fn get_attr(name: &str, attrs: &Vec<Attribute>) -> Option<String> {
    for attr in attrs {
        if attr.name.local.as_ref() == name {
            return Some(attr.value.to_string());
        }
    }

    None
}

pub fn get_attrs(names: Vec<&str>, attrs: &Vec<Attribute>) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for attr in attrs {
        if names.contains(&attr.name.local.as_ref()) {
            map.insert(attr.name.local.as_ref().to_string(), attr.value.to_string());
        }
    }

    map
}

pub fn create_element(dom: &mut RcDom, name: &str, attrs: Vec<Attribute>) -> Rc<Node> {
    let name = QualName::new(None, ns!(), LocalName::from(name));

    dom.create_element(name, attrs, ElementFlags::default())
}

pub fn get_text_len(handle: Handle) -> usize {
    let mut len = 0;

    for child in handle.children.borrow().iter() {
        match child.data {
            Text { ref contents } => {
                len += contents.borrow().len();
            }
            Element { .. } => {
                len += get_text_len(child.clone());
            }
            _ => {}
        }
    }

    len
}

pub fn get_tag_name(handle: Handle) -> Option<String> {
    match handle.data {
        Document => Some("document".to_string()),
        Element { ref name, .. } => Some(name.local.as_ref().to_lowercase().to_string()),
        _ => None,
    }
}

pub fn set_attr(handle: Handle, name: &str, value: &str) {
    match handle.data {
        Element {
            name: _, ref attrs, ..
        } => {
            let attrs = &mut attrs.borrow_mut();
            if let Some(index) = attrs
                .iter()
                .position(|attr| attr.name.local.as_ref() == name)
            {
                match StrTendril::from_str(value) {
                    Ok(value) => {
                        attrs[index] = Attribute {
                            name: attrs[index].name.clone(),
                            value: value,
                        }
                    }
                    Err(_) => (),
                }
            } else {
                attrs.push(Attribute {
                    name: QualName::new(None, ns!(), LocalName::from(name)),
                    value: value.into(),
                });
            }
        }
        _ => {}
    }
}

// @FIXME match text node as parameter
pub fn get_text_children_count(handle: Handle) -> usize {
    let mut count = 0;
    for child in handle.children.borrow().iter() {
        let c = child.clone();
        match c.data {
            Text { ref contents } => {
                let s = contents.borrow();
                if s.trim().len() >= 20 {
                    count += 1
                }
            }
            _ => (),
        }
    }

    count
}

pub fn get_attr_by_name<'a>(handle: Handle, name: &str) -> Option<String> {
    match handle.data {
        Element {
            name: _, ref attrs, ..
        } => get_attr(name, &attrs.borrow()),
        _ => None,
    }
}

pub fn fix_img_path(handle: Handle, url: &Url) -> bool {
    let src = get_attr_by_name(handle.clone(), "src");
    if src.is_none() {
        return false;
    }
    let s = src.unwrap();
    if !s.starts_with("//") && !s.starts_with("http://") && s.starts_with("https://") {
        match url.join(&s) {
            Ok(new_url) => set_attr(handle, "src", new_url.as_str()),
            Err(_) => (),
        }
    }
    true
}

#[cfg(test)]
mod test {
    pub use super::*;

    #[test]
    fn it_returns_title_element() {
        let html = r#"<html><head><title>My Document</title></head><body><h1>Hello, world!</h1></body></html>"#;
        let dom = parse_from(&mut html.as_bytes()).unwrap();
        let mut nodes = vec![];

        let result = query_by_tagname(dom.document.clone(), "title", &mut nodes);

        assert_eq!(result, true);

        let mut title = String::new();

        get_text_content(nodes[0].clone(), &mut title);
        assert_eq!(title, "My Document");
    }

    #[test]
    fn it_returs_attrs() {
        let html =
            r#"<html><body><h1 id="header" class="header_type1">Hello, world!</h1></body></html>"#;
        let dom = parse_from(&mut html.as_bytes()).unwrap();
        let mut nodes = vec![];

        let result = query_by_tagname(dom.document.clone(), "h1", &mut nodes);
        assert_eq!(result, true);

        match nodes[0].clone().data {
            Element {
                ref name,
                ref attrs,
                ..
            } => {
                let attrs = attrs.borrow_mut();
                assert_eq!(name.local.as_ref(), "h1");
                assert_eq!(attrs.len(), 2);
                assert_eq!(attrs[0].name.local.as_ref(), "id");
                assert_eq!(attrs[0].value.to_string(), "header");
                assert_eq!(attrs[1].name.local.as_ref(), "class");
                assert_eq!(attrs[1].value.to_string(), "header_type1");

                assert_eq!(get_attr("id", &attrs).unwrap(), "header");
                assert_eq!(get_attr("class", &attrs).unwrap(), "header_type1");

                let attrs = get_attrs(vec!["id", "class"], &attrs);
                assert_eq!(attrs.len(), 2);
                assert_eq!(attrs.get("id").unwrap(), "header");
                assert_eq!(attrs.get("class").unwrap(), "header_type1");
            }
            _ => {}
        }
    }

    #[test]
    fn it_returns_element() {
        let node = create_element(&mut RcDom::default(), "p", vec![]);

        match node.data {
            Element { ref name, .. } => {
                assert_eq!(name.local.as_ref(), "p");
            }
            _ => {}
        }
    }

    #[test]
    fn it_returns_info() {
        let html =
            r#"<html><body><h1 id="header" class="header_type1">Hello, world!</h1></body></html>"#;
        let dom = parse_from(&mut html.as_bytes()).unwrap();

        let mut nodes = vec![];

        query_by_tagname(dom.document.clone(), "h1", &mut nodes);

        let text_size = get_text_len(nodes[0].clone());

        assert_eq!(text_size, 13);

        let tag_name = get_tag_name(nodes[0].clone());

        assert_eq!(tag_name.unwrap(), "h1");
    }
}
