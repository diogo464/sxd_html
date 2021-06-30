use html5ever::QualName;
use sxd_document::dom::{
    ChildOfElement, ChildOfRoot, Comment, Element, ParentOfChild, ProcessingInstruction, Root, Text,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Handle<'d> {
    Document(Root<'d>),
    Element(Element<'d>, QualName),
    Comment(Comment<'d>),
    ProcessingInstruction(ProcessingInstruction<'d>),
    Text(Text<'d>),
}

impl<'d> Handle<'d> {
    pub fn element_ref(&self) -> &Element<'d> {
        match self {
            Self::Element(e, _) => e,
            _ => panic!("Handle is not an element"),
        }
    }

    pub fn parent(&self) -> Option<ParentOfChild<'d>> {
        match self {
            Self::Document(_) => panic!("Cannot call parent on Document"),
            Self::Element(e, _) => e.parent(),
            Self::Comment(c) => c.parent(),
            Self::ProcessingInstruction(p) => p.parent(),
            Self::Text(t) => t.parent().map(ParentOfChild::Element),
        }
    }

    pub fn following_siblings(&self) -> Vec<ChildOfElement<'d>> {
        match self {
            Self::Document(_) => panic!("Cannot call following_siblings on Document"),
            Self::Element(e, _) => e.following_siblings(),
            Self::Comment(c) => c.following_siblings(),
            Self::ProcessingInstruction(p) => p.following_siblings(),
            Self::Text(t) => t.following_siblings(),
        }
    }

    pub fn remove_from_parent(&self) {
        match self {
            Self::Document(_) => panic!("Cannot call remove_from_parent on Document"),
            Self::Element(e, _) => e.remove_from_parent(),
            Self::Comment(c) => c.remove_from_parent(),
            Self::ProcessingInstruction(p) => p.remove_from_parent(),
            Self::Text(t) => t.remove_from_parent(),
        }
    }
}

impl<'d> From<Handle<'d>> for ChildOfRoot<'d> {
    fn from(h: Handle<'d>) -> Self {
        match h {
            Handle::Document(_) => panic!("Handle::Document cannot be made into ChildOfRoot"),
            Handle::Element(x, _) => x.into(),
            Handle::Comment(x) => x.into(),
            Handle::ProcessingInstruction(x) => x.into(),
            Handle::Text(_) => panic!("Handle::Text cannot be mande into ChildOfRoot"),
        }
    }
}

impl<'d> From<Handle<'d>> for ChildOfElement<'d> {
    fn from(h: Handle<'d>) -> Self {
        ChildOfRoot::from(h).into()
    }
}

impl<'d> From<Element<'d>> for Handle<'d> {
    fn from(e: Element<'d>) -> Self {
        let qualname = super::qualname_from_qname(e.name());
        Self::Element(e, qualname)
    }
}

impl<'d> From<ParentOfChild<'d>> for Handle<'d> {
    fn from(p: ParentOfChild<'d>) -> Self {
        match p {
            ParentOfChild::Root(r) => Self::Document(r),
            ParentOfChild::Element(e) => Self::from(e),
        }
    }
}

impl<'d> From<ChildOfElement<'d>> for Handle<'d> {
    fn from(c: ChildOfElement<'d>) -> Self {
        match c {
            ChildOfElement::Element(x) => Self::from(x),
            ChildOfElement::Text(x) => Handle::Text(x),
            ChildOfElement::Comment(x) => Handle::Comment(x),
            ChildOfElement::ProcessingInstruction(x) => Handle::ProcessingInstruction(x),
        }
    }
}
