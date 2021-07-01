use sxd_xpath::{nodeset::Node, Context, Error, Factory, Value};

fn main() -> anyhow::Result<()> {
    let contents = reqwest::blocking::get("https://github.com/trending")?.text()?;
    let package = sxd_html::parse_html(&contents);
    let document = package.as_document();

    let mut trending_repos: Vec<String> = Default::default();
    let repo_as = match evaluate_xpath_node(document.root(), "//article/h1/a")? {
        Value::Nodeset(set) => set,
        _ => panic!("Expected node set"),
    }
    .document_order();

    for repo_a in repo_as {
        let user = evaluate_xpath_node(repo_a, "./span/text()")?.into_string();
        let name = repo_a
            .children()
            .last()
            .unwrap()
            .text()
            .map(|t| t.text())
            .unwrap_or_default();
        trending_repos.push(format!("{}{}", user.trim(), name.trim()));
    }

    println!("Trending Repos :");
    for name in trending_repos {
        println!("\t{}", name);
    }

    Ok(())
}

fn evaluate_xpath_node<'d>(node: impl Into<Node<'d>>, expr: &str) -> Result<Value<'d>, Error> {
    let factory = Factory::new();
    let expression = factory.build(expr)?;
    let expression = expression.ok_or(Error::NoXPath)?;

    let context = Context::new();

    expression
        .evaluate(&context, node.into())
        .map_err(Into::into)
}
