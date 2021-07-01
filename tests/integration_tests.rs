#[test]
fn parse_simple() {
    let contents = "<!DOCTYPE html><html><div>hello<br>bye</div></html>";
    let (package, errors) = sxd_html::parse_html_with_errors(contents);
    assert_eq!(0, errors.len());

    let root = package.as_document().root();
    let root = root.children()[0]
        .element()
        .expect("html should be root element");

    assert_eq!("html", root.name().local_part());

    let children = root.children();
    // head and body are added if not present
    assert_eq!(2, children.len());

    let head = children[0].element().unwrap();
    let body = children[1].element().unwrap();

    assert_eq!("head", head.name().local_part());
    assert_eq!(0, head.children().len());

    let children = body.children();
    assert_eq!("body", body.name().local_part());
    assert_eq!(1, children.len());

    let c0 = children[0].element().unwrap();
    let children = c0.children();
    assert_eq!("div", c0.name().local_part());
    assert_eq!(3, children.len());

    let c0 = children[0].text().unwrap();
    let c1 = children[1].element().unwrap();
    let c2 = children[2].text().unwrap();

    assert_eq!("hello", c0.text());

    assert_eq!("br", c1.name().local_part());
    assert_eq!(0, c1.children().len());

    assert_eq!("bye", c2.text());
}
