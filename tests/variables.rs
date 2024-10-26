use crate::common::interpreter;

mod common;

#[test]
fn run_var_print_1() {
    let (output, err) = interpreter::run_content(
        r#"var a = "foo";
print a;"#,
    );
    assert_none!(err);
    assert_eq!("foo\n", output);
}

#[test]
fn run_var_print_2() {
    let (output, err) = interpreter::run_content(
        r#"var a = 20 + 22;
print a;"#,
    );
    assert_none!(err);
    assert_eq!("42\n", output);
}

#[test]
fn run_var_redeclare_print_2() {
    let (output, err) = interpreter::run_content(
        r#"var a = 20 + 22;
var a = "foo";
print a;"#,
    );
    assert_none!(err);
    assert_eq!("foo\n", output);
}

#[test]
fn run_var_unassigned() {
    let (output, err) = interpreter::run_content(
        r#"var a;
print a;"#,
    );
    assert_none!(err);
    assert_eq!("nil\n", output);
}

#[test]
fn run_var_undefined() {
    let (_, err) = interpreter::run_content("print a;");
    assert_some!(err);
}

#[test]
fn run_var_assignment_1() {
    let (output, err) = interpreter::run_content(
        r#"var a = 20;
a = 42;
print a;"#,
    );
    assert_none!(err);
    assert_eq!("42\n", output);
}

#[test]
fn run_var_assignment_2() {
    let (output, err) = interpreter::run_content(
        r#"var a = 20;
a = a + 22;
print a;"#,
    );
    assert_none!(err);
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
    assert_none!(err);
    assert_eq!(
        r#"1
2
2
"#,
        output
    );
}
