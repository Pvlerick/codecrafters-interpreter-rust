use crate::common::interpreter;

mod common;

#[test]
fn run_print_string() {
    let (output, err) = interpreter::run_content("print \"foo\";");
    assert!(err.is_none());
    assert_eq!("foo\n", output);
}

#[test]
fn run_print_true() {
    let (output, err) = interpreter::run_content("print true;");
    assert!(err.is_none());
    assert_eq!("true\n", output);
}

#[test]
fn run_print_boolean_comparison() {
    // let (output, err) = interpreter::run_content("print false != true;");
    let (output, err) = interpreter::run_content("print true != false;");
    assert!(err.is_none());
    assert_eq!("true\n", output);
}

#[test]
fn run_print_multiple_statements() {
    let (output, err) = interpreter::run_content("print \"foo\"; print 42;");
    assert!(err.is_none());
    assert_eq!(
        r#"foo
42
"#,
        output
    );
}

#[test]
fn run_print_multiple_lines_1() {
    let (output, err) = interpreter::run_content(
        r#"print "foo";
print 42;"#,
    );
    assert!(err.is_none());
    assert_eq!(
        r#"foo
42
"#,
        output
    );
}

#[test]
fn run_print_multiple_lines_2() {
    let (output, err) = interpreter::run_content(
        r#"
print true != true;

print "36
10
78
";

print "There should be an empty line above this.";"#,
    );
    assert!(err.is_none());
    assert_eq!(
        r#"false
36
10
78

There should be an empty line above this.
"#,
        output
    );
}

#[test]
fn run_expression_statements() {
    let (output, err) = interpreter::run_content(
        r#"(85 + 64 - 59) > (30 - 85) * 2;
print !false;
"world" + "hello" + "foo" + "quz" == "worldhellofooquz";
print !false;"#,
    );
    assert!(err.is_none());
    assert_eq!(
        r#"true
true
"#,
        output
    );
}

#[test]
fn run_var_print_1() {
    let (output, err) = interpreter::run_content(
        r#"var a = "foo";
print a;"#,
    );
    assert!(err.is_none());
    assert_eq!("foo\n", output);
}

#[test]
fn run_var_print_2() {
    let (output, err) = interpreter::run_content(
        r#"var a = 20 + 22;
print a;"#,
    );
    assert!(err.is_none());
    assert_eq!("42\n", output);
}

#[test]
fn run_var_redeclare_print_2() {
    let (output, err) = interpreter::run_content(
        r#"var a = 20 + 22;
var a = "foo";
print a;"#,
    );
    assert!(err.is_none());
    assert_eq!("foo\n", output);
}

#[test]
fn run_var_unassigned() {
    let (output, err) = interpreter::run_content(
        r#"var a;
print a;"#,
    );
    assert!(err.is_none());
    assert_eq!("nil\n", output);
}

#[test]
fn run_var_undefined() {
    let (_, err) = interpreter::run_content("print a;");
    assert!(err.is_some());
}

#[test]
fn run_var_assignment_1() {
    let (output, err) = interpreter::run_content(
        r#"var a = 20;
a = 42;
print a;"#,
    );
    assert!(err.is_none());
    assert_eq!("42\n", output);
}

#[test]
fn run_var_assignment_2() {
    let (output, err) = interpreter::run_content(
        r#"var a = 20;
a = a + 22;
print a;"#,
    );
    assert!(err.is_none());
    assert_eq!("42\n", output);
}

#[test]
fn run_var_assignment_3() {
    let (output, err) = interpreter::run_content(
        r#"var quz;
quz = 1;
print quz;
print quz = 2;
print quz;
"#,
    );
    assert!(err.is_none());
    assert_eq!(
        r#"1
2
2
"#,
        output
    );
}

#[test]
fn run_blocks_1() {
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
fn run_blocks_2() {
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
fn run_blocks_3() {
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
fn run_blocks_4() {
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
fn run_blocks_5() {
    let (_, err) = interpreter::run_content(
        r#"{
     var foo = 11;
}
print foo;"#,
    );
    assert!(err.is_some());
}

#[test]
fn run_blocks_6() {
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
fn run_blocks_7() {
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
