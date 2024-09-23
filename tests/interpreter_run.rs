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
