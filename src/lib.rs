mod error;
mod handle;
mod util;

pub use error::Error;
pub(crate) use handle::Handle;

use html5ever::{
    tendril::Tendril,
    tree_builder::{NodeOrText, TreeSink},
};
use sxd_document::{
    dom::{ChildOfElement, Document},
    Package,
};

#[derive(Debug)]
struct DocHtmlSink<'d> {
    document: Document<'d>,
    document_handle: Handle<'d>,
    errors: Vec<Error>,
    current_line: u64,
}

impl<'d> DocHtmlSink<'d> {
    fn new(document: Document<'d>) -> Self {
        let document_handle = Handle::Document(document.root());

        Self {
            document,
            document_handle,
            errors: Default::default(),
            current_line: 0,
        }
    }
}

impl<'d> TreeSink for DocHtmlSink<'d> {
    type Handle = Handle<'d>;
    type Output = Vec<Error>;

    fn set_current_line(&mut self, line_number: u64) {
        self.current_line = line_number;
    }

    fn finish(self) -> Self::Output {
        self.errors
    }

    fn parse_error(&mut self, msg: std::borrow::Cow<'static, str>) {
        self.errors.push(Error::new(self.current_line, msg));
    }

    fn get_document(&mut self) -> Self::Handle {
        self.document_handle.clone()
    }

    // this is only called on elements
    fn elem_name<'h>(&'h self, target: &'h Self::Handle) -> html5ever::ExpandedName<'h> {
        match target {
            Handle::Element(_, qualname) => qualname.expanded(),
            _ => panic!("not an element"),
        }
    }

    fn create_element(
        &mut self,
        name: html5ever::QualName,
        attrs: Vec<html5ever::Attribute>,
        _flags: html5ever::tree_builder::ElementFlags,
    ) -> Self::Handle {
        let qname = util::qualname_as_qname(&name);
        let elem = self.document.create_element(qname);

        for attr in attrs {
            let qname = util::qualname_as_qname(&attr.name);
            elem.set_attribute_value(qname, attr.value.as_ref());
        }

        Handle::Element(elem, name)
    }

    fn create_comment(&mut self, text: html5ever::tendril::StrTendril) -> Self::Handle {
        let comment = self.document.create_comment(text.as_ref());
        Handle::Comment(comment)
    }

    fn create_pi(
        &mut self,
        target: html5ever::tendril::StrTendril,
        data: html5ever::tendril::StrTendril,
    ) -> Self::Handle {
        let data = if data.is_empty() {
            None
        } else {
            Some(data.as_ref())
        };

        let pi = self
            .document
            .create_processing_instruction(target.as_ref(), data);

        Handle::ProcessingInstruction(pi)
    }

    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        match parent {
            Handle::Document(root) => {
                // text cant be appended to root so no need to concatenate it
                let child = util::node_or_text_into_child_of_root(child);
                root.append_child(child);
            }
            Handle::Element(elem, _) => {
                let last = elem.children().into_iter().last();

                match (last, child) {
                    (Some(ChildOfElement::Text(x)), NodeOrText::AppendText(y)) => {
                        let mut new_text = x.text().to_string();
                        new_text.push_str(y.as_ref());
                        x.set_text(&new_text);
                    }
                    (_, child) => {
                        let document = elem.document();
                        let child = util::node_or_text_into_child_of_element(&document, child);
                        elem.append_child(child);
                    }
                }
            }
            _ => panic!("Can only appent into document or element"),
        }
    }

    fn append_based_on_parent_node(
        &mut self,
        _element: &Self::Handle,
        _prev_element: &Self::Handle,
        _child: html5ever::tree_builder::NodeOrText<Self::Handle>,
    ) {
        // I dont understand this one
        unimplemented!()
    }

    fn append_doctype_to_document(
        &mut self,
        _name: html5ever::tendril::StrTendril,
        _public_id: html5ever::tendril::StrTendril,
        _system_id: html5ever::tendril::StrTendril,
    ) {
        // ignored, cant seem to find a way to add doctype using sxd_document
    }

    fn get_template_contents(&mut self, _target: &Self::Handle) -> Self::Handle {
        // dont understand this
        // it seems to just return the document
        // https://github.com/servo/html5ever/blob/master/rcdom/lib.rs#L232
        unimplemented!()
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&mut self, _mode: html5ever::tree_builder::QuirksMode) {
        // ignored
    }

    fn append_before_sibling(
        &mut self,
        sibling: &Self::Handle,
        new_node: NodeOrText<Self::Handle>,
    ) {
        let parent = sibling.parent().expect("must have a parent");

        let children = {
            let mut v = vec![ChildOfElement::from(sibling.clone())];
            v.extend(sibling.following_siblings().into_iter());
            v
        };

        for child in children.iter() {
            util::child_of_element_remove_from_parent(child);
        }

        util::parent_of_child_append_node_or_text(&parent, new_node);
        let parent_handle = Handle::from(parent);
        for child in children {
            let node_or_text = match child {
                ChildOfElement::Text(t) => NodeOrText::AppendText(Tendril::from(t.text())),
                coe => NodeOrText::AppendNode(Handle::from(coe)),
            };
            self.append(&parent_handle, node_or_text);
        }
    }

    // this is only called on elements
    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        let elem = target.element_ref();
        for attr in attrs {
            let qname = util::qualname_as_qname(&attr.name);
            if elem.attribute_value(qname).is_some() {
                continue;
            }

            elem.set_attribute_value(qname, attr.value.as_ref());
        }
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        target.remove_from_parent();
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        let node = node.element_ref();
        let new_parent = new_parent.element_ref();

        let children = node.children();
        node.clear_children();
        new_parent.append_children(children);
    }
}

pub fn parse_html(contents: &str) -> Package {
    parse_html_with_errors(contents).0
}

pub fn parse_html_with_errors(contents: &str) -> (Package, Vec<Error>) {
    use html5ever::driver::ParseOpts;
    use html5ever::tendril::TendrilSink;
    use html5ever::tree_builder::TreeBuilderOpts;

    let package = Package::new();
    let document = package.as_document();
    let sink = DocHtmlSink::new(document);

    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            exact_errors: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let parser = html5ever::parse_document(sink, opts);
    let errors = parser.one(contents);

    (package, errors)
}
