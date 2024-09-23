use crate::common::interpreter;

mod common;

#[test]
fn run_block() {
    let (output, err) = interpreter::run_content(
        r#"{
    var hello = "baz";
    print hello;
}"#,
    );
    assert!(err.is_none());
    assert_eq!("baz\n", output);
}

#[test]
fn run_two_blocks() {
    let (output, err) = interpreter::run_content(
        r#"{
    var world = "before";
    print world;
}
{
    var world = "after";
    print world;
}"#,
    );
    assert!(err.is_none());
    assert_eq!(
        r#"before
after
"#,
        output
    );
}

#[test]
fn run_block_scoped_variable() {
    let (output, err) = interpreter::run_content(
        r#"{
    var hello = 88;
    {
        var foo = hello;
        print foo;
    }
    print hello;
}"#,
    );
    assert!(err.is_none());
    assert_eq!(
        r#"88
88
"#,
        output
    );
}

#[test]
fn run_block_scoped_variable_and_assignment() {
    let (output, err) = interpreter::run_content(
        r#"{
    var foo = 88;
    {
        var foo = 42;
        print foo;
    }
    print foo;
}"#,
    );
    assert!(err.is_none());
    assert_eq!(
        r#"42
88
"#,
        output
    );
}

#[test]
fn run_block_var_scope_error() {
    let (_, err) = interpreter::run_content(
        r#"{
     var foo = 11;
}
print foo;"#,
    );
    assert!(err.is_some());
}

#[test]
fn run_block_unclosed_block() {
    let (_, err) = interpreter::run_content(
        r#"{
    var quz = 73;
    var bar = 73;
    {
        print quz + bar;
}
"#,
    );

    assert!(err.is_some());
}

#[test]
fn run_block_assign_using_same_name_as_outer_variable() {
    let (output, err) = interpreter::run_content(
        r#"var a = 1;
{
    var a = a + 2;
    print a;
}
print a;"#,
    );
    assert!(err.is_none());
    assert_eq!(
        r#"3
1
"#,
        output
    );
}
