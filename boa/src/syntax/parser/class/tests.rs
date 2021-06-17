use crate::syntax::{
    ast::node::{ClassDecl, Declaration, DeclarationList, FunctionDecl},
    parser::{tests::check_parser, Parser},
};

/// Checks for empty class parsing (making sure the keyword works)
#[test]
fn check_empty() {
    check_parser(
        "class Empty {}",
        vec![ClassDecl::new(Box::from("Empty"), None, vec![]).into()],
    );
}

/// Checks for a constructor being parsed in a class
#[test]
fn check_basic() {
    check_parser(
        r#"
        class Basic {
            constructor() {
                let val;
            }
        }
        "#,
        vec![ClassDecl::new(
            Box::from("Basic"),
            Some(FunctionDecl::new(
                Box::from("constructor"),
                vec![],
                vec![
                    DeclarationList::Let(vec![Declaration::new("val", None)].into_boxed_slice())
                        .into(),
                ],
            )),
            vec![],
        )
        .into()],
    );
}

/// Checks for multiple functions being parsed.
#[test]
fn check_multi() {
    check_parser(
        r#"
        class Multi {
            constructor() {
                let val;
            }
            say_hello() {}
            say_hello_again() {}
        }
        "#,
        vec![ClassDecl::new(
            Box::from("Multi"),
            Some(FunctionDecl::new(
                Box::from("constructor"),
                vec![],
                vec![
                    DeclarationList::Let(vec![Declaration::new("val", None)].into_boxed_slice())
                        .into(),
                ],
            )),
            vec![
                FunctionDecl::new(Box::from("say_hello"), vec![], vec![]),
                FunctionDecl::new(Box::from("say_hello_again"), vec![], vec![]),
            ],
        )
        .into()],
    );
}

/// Checks for multiple constructors being a parse error.
#[test]
fn check_multi_constructors() {
    let js = r#"
        class InvalidBecauseConstructors {
            constructor() {
            }
            constructor() {
            }
        }
        "#;
    assert!(Parser::new(js.as_bytes(), false).parse_all().is_err());
}
