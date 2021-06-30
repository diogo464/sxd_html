use html5ever::{tree_builder::NodeOrText, QualName};
use sxd_document::{
    dom::{ChildOfElement, ChildOfRoot, Document, ParentOfChild},
    QName,
};

use crate::Handle;

pub fn qualname_from_qname(qname: QName) -> QualName {
    QualName::new(
        None,
        qname.namespace_uri().unwrap_or_default().into(),
        qname.local_part().into(),
    )
}

pub fn qualname_as_qname(qualname: &QualName) -> QName {
    let ns = if qualname.ns.is_empty() {
        None
    } else {
        Some(qualname.ns.as_ref())
    };
    QName::with_namespace_uri(ns, qualname.local.as_ref())
}

pub fn node_or_text_into_child_of_root(node_or_text: NodeOrText<Handle>) -> ChildOfRoot {
    match node_or_text {
        NodeOrText::AppendNode(handle) => ChildOfRoot::from(handle),
        NodeOrText::AppendText(_) => panic!("Text cannot be made into ChildOfRoot"),
    }
}

pub fn node_or_text_into_child_of_element<'d>(
    document: &Document<'d>,
    node_or_text: NodeOrText<Handle<'d>>,
) -> ChildOfElement<'d> {
    match node_or_text {
        NodeOrText::AppendNode(handle) => ChildOfElement::from(handle),
        NodeOrText::AppendText(text) => ChildOfElement::from(document.create_text(text.as_ref())),
    }
}

pub fn child_of_element_remove_from_parent(coe: &ChildOfElement) {
    match coe {
        ChildOfElement::Element(x) => x.remove_from_parent(),
        ChildOfElement::Text(x) => x.remove_from_parent(),
        ChildOfElement::Comment(x) => x.remove_from_parent(),
        ChildOfElement::ProcessingInstruction(x) => x.remove_from_parent(),
    }
}

pub fn parent_of_child_append_node_or_text(poc: &ParentOfChild, noe: NodeOrText<Handle>) {
    match poc {
        ParentOfChild::Root(r) => r.append_child(node_or_text_into_child_of_root(noe)),
        ParentOfChild::Element(e) => {
            e.append_child(node_or_text_into_child_of_element(&e.document(), noe))
        }
    }
}
