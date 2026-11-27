mod error;
mod lexer;
mod parser;
mod type_checker;
use std::fs;

fn main() -> anyhow::Result<()> {
    let content = fs::read_to_string("examples/add.mini")?;

    let tokens = lexer::tokenize(&content)?;

    let ast = parser::parse(tokens)?;

    println!("{ast:?}");
    println!("================================================");

    let typed_ast = type_checker::check(&ast);

    println!("{typed_ast:#?}");

    Ok(())
}
