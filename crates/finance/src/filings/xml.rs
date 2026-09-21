//! Reading an ePorezna form's tree without knowing the form.
//!
//! Every form is elements with text in them: scalars (`Podatak1`), pairs
//! (`Podatak200/{Vrijednost, Porez}`), and containers of repeated blocks
//! (`Primatelji/Primatelj`, `Isporuke/Isporuka`, `StranaB/Primatelji/P`).
//! [`flatten`] walks a subtree once and writes scalars as `path -> text`
//! and repeated blocks as row objects, so a form's own module only says
//! which containers hold rows and which keys matter. Names are matched by
//! local name: the official examples disagree on which namespace a child
//! sits in, and a real download may differ again.

use std::collections::BTreeMap;

use roxmltree::Node;
use serde_json::{Map, Value};

/// The `xsi` namespace, for `nil`.
const XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";

/// The element children of `n`, in document order.
pub fn elements<'a, 'i>(n: Node<'a, 'i>) -> impl Iterator<Item = Node<'a, 'i>> {
    n.children().filter(Node::is_element)
}

/// The first element child of `n` with this local name.
#[must_use]
pub fn child<'a, 'i>(n: Node<'a, 'i>, local: &str) -> Option<Node<'a, 'i>> {
    elements(n).find(|c| c.tag_name().name() == local)
}

/// The element at this local-name path below `n`.
#[must_use]
pub fn find<'a, 'i>(n: Node<'a, 'i>, path: &[&str]) -> Option<Node<'a, 'i>> {
    path.iter().try_fold(n, |at, local| child(at, local))
}

/// The trimmed text of the child `local`, if present and not empty.
#[must_use]
pub fn text(n: Node<'_, '_>, local: &str) -> Option<String> {
    child(n, local)
        .and_then(|c| content(c))
        .filter(|s| !s.is_empty())
}

/// The trimmed text of `n` itself; empty for a `xsi:nil` element.
#[must_use]
pub fn content(n: Node<'_, '_>) -> Option<String> {
    if n.attribute((XSI, "nil"))
        .is_some_and(|v| v == "true" || v == "1")
    {
        return Some(String::new());
    }
    n.text().map(|t| t.trim().to_owned())
}

/// The key a local name becomes: `Podatak1` is `1`, `Vrijednost` stays.
#[must_use]
pub fn key_name(local: &str) -> &str {
    local.strip_prefix("Podatak").unwrap_or(local)
}

fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    }
}

fn joined_path(rel: &str, local: &str) -> String {
    if rel.is_empty() {
        local.to_owned()
    } else {
        format!("{rel}/{local}")
    }
}

/// An element with no element children.
fn xml_leaf(n: Node<'_, '_>) -> bool {
    elements(n).next().is_none()
}

/// Whether two element children share a local name: the shape of a row
/// container.
fn repeated(kids: &[Node<'_, '_>]) -> bool {
    kids.iter().enumerate().any(|(i, a)| {
        kids[..i]
            .iter()
            .any(|b| b.tag_name().name() == a.tag_name().name())
    })
}

/// Walk the subtree below `node`. A leaf becomes `values[prefix.name]`; a
/// container whose local-name path (relative to the walk's start) is in
/// `row_paths`, or whose children repeat a name, becomes one row object per
/// child in `rows`, tagged `_kind` with the container's key; anything else
/// recurses. Attributes are ignored.
pub fn flatten(
    node: Node<'_, '_>,
    prefix: &str,
    rel: &str,
    row_paths: &[&str],
    values: &mut BTreeMap<String, String>,
    rows: &mut Vec<Value>,
) {
    for c in elements(node) {
        let local = c.tag_name().name();
        let key = join(prefix, key_name(local));
        let path = joined_path(rel, local);
        let kids: Vec<Node<'_, '_>> = elements(c).collect();
        if kids.is_empty() {
            values.insert(key, content(c).unwrap_or_default());
        } else if row_paths.contains(&path.as_str()) || repeated(&kids) {
            // The rows are the children named like the first one; a block
            // with another name beside them (PD-IPO's `Sveukupno` inside
            // `Osobe`) is a total and flattens to values under the container.
            let row_name = kids[0].tag_name().name();
            for k in kids {
                if k.tag_name().name() != row_name {
                    let sub_key = join(&key, key_name(k.tag_name().name()));
                    let sub_path = joined_path(&path, k.tag_name().name());
                    if xml_leaf(k) {
                        values.insert(sub_key, content(k).unwrap_or_default());
                    } else {
                        flatten(k, &sub_key, &sub_path, row_paths, values, rows);
                    }
                    continue;
                }
                let mut row = match object(k) {
                    Value::Object(m) => m,
                    other => {
                        let mut m = Map::new();
                        m.insert("_value".into(), other);
                        m
                    }
                };
                row.insert("_kind".into(), Value::String(key.clone()));
                rows.push(Value::Object(row));
            }
        } else {
            flatten(c, &key, &path, row_paths, values, rows);
        }
    }
}

/// An element as JSON: text for a leaf, an object for a block, an array
/// where children repeat a name.
#[must_use]
pub fn object(node: Node<'_, '_>) -> Value {
    let kids: Vec<Node<'_, '_>> = elements(node).collect();
    if kids.is_empty() {
        return Value::String(content(node).unwrap_or_default());
    }
    let mut out = Map::new();
    for k in kids {
        let name = key_name(k.tag_name().name()).to_owned();
        let v = object(k);
        match out.get_mut(&name) {
            Some(Value::Array(a)) => a.push(v),
            Some(existing) => {
                let first = existing.take();
                *existing = Value::Array(vec![first, v]);
            }
            None => {
                out.insert(name, v);
            }
        }
    }
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(xml: &str) -> roxmltree::Document<'_> {
        roxmltree::Document::parse(xml).expect("well-formed")
    }

    #[test]
    fn leaves_pairs_and_rows_take_their_shapes() {
        let d = doc(
            "<T xmlns=\"urn:x\"><Podatak1>1.00</Podatak1><Podatak200><Vrijednost>2.00</Vrijednost><Porez>0.50</Porez></Podatak200>\
             <Isporuke><Isporuka><RedBr>1</RedBr><I1>3.00</I1></Isporuka><Isporuka><RedBr>2</RedBr><I1>4.00</I1></Isporuka></Isporuke>\
             <Racuni><Racun><BrojRacuna>HR12</BrojRacuna></Racun></Racuni></T>",
        );
        let mut values = BTreeMap::new();
        let mut rows = Vec::new();
        flatten(
            d.root_element(),
            "",
            "",
            &["Racuni"],
            &mut values,
            &mut rows,
        );
        assert_eq!(values["1"], "1.00");
        assert_eq!(values["200.Vrijednost"], "2.00");
        assert_eq!(values["200.Porez"], "0.50");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["_kind"], "Isporuke");
        assert_eq!(rows[1]["RedBr"], "2");
        assert_eq!(rows[2]["_kind"], "Racuni");
        assert_eq!(rows[2]["BrojRacuna"], "HR12");
    }

    #[test]
    fn nil_is_empty_and_namespaces_do_not_matter() {
        let d = doc(
            "<a:T xmlns:a=\"urn:a\" xmlns:b=\"urn:b\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\
             <b:Podatak5 xsi:nil=\"true\"/><b:Podatak6> 7.00 </b:Podatak6></a:T>",
        );
        let mut values = BTreeMap::new();
        let mut rows = Vec::new();
        flatten(d.root_element(), "", "", &[], &mut values, &mut rows);
        assert_eq!(values["5"], "");
        assert_eq!(values["6"], "7.00");
        assert!(rows.is_empty());
        assert_eq!(text(d.root_element(), "Podatak6").as_deref(), Some("7.00"));
        assert!(find(d.root_element(), &["Podatak6"]).is_some());
    }

    #[test]
    fn a_total_beside_the_rows_is_a_value_not_a_row() {
        let d = doc(
            "<T><Podaci2><Osobe><Osoba><O1>1</O1><Potrazivanja><Potrazivanje><P2>10.00</P2></Potrazivanje></Potrazivanja></Osoba>\
             <Sveukupno><S1>10.00</S1><S4>0.20</S4></Sveukupno></Osobe></Podaci2></T>",
        );
        let mut values = BTreeMap::new();
        let mut rows = Vec::new();
        flatten(
            d.root_element(),
            "",
            "",
            &["Podaci2/Osobe"],
            &mut values,
            &mut rows,
        );
        assert_eq!(rows.len(), 1, "one person, one row");
        assert_eq!(rows[0]["_kind"], "Podaci2.Osobe");
        assert_eq!(rows[0]["Potrazivanja"]["Potrazivanje"]["P2"], "10.00");
        assert_eq!(values["Podaci2.Osobe.Sveukupno.S1"], "10.00");
        assert_eq!(values["Podaci2.Osobe.Sveukupno.S4"], "0.20");
    }

    #[test]
    fn object_nests_and_arrays_repeats() {
        let d = doc("<O><A>1</A><B><C>2</C><C>3</C></B></O>");
        let v = object(d.root_element());
        assert_eq!(v["A"], "1");
        assert_eq!(v["B"]["C"][1], "3");
    }
}
