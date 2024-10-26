use crate::common::interpreter;

mod common;

#[test]
fn run_print_string() {
    let (output, err) = interpreter::run_content("print \"foo\";");
    assert_none!(err);
    assert_eq!("foo\n", output);
}

#[test]
fn run_print_true() {
    let (output, err) = interpreter::run_content("print true;");
    assert_none!(err);
    assert_eq!("true\n", output);
}

#[test]
fn run_print_boolean_comparison() {
    let (output, err) = interpreter::run_content("print true != false;");
    assert_none!(err);
    assert_eq!("true\n", output);
}

#[test]
fn run_print_multiple_statements() {
    let (output, err) = interpreter::run_content("print \"foo\"; print 42;");
    assert_none!(err);
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
    assert_none!(err);
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
    assert_none!(err);
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
