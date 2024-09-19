use std::io::sink;

use interpreter_starter_rust::interpreter::Interpreter;

use crate::common::TempFile;

mod common;

#[test]
fn run_print_string() {
    let mut tmp_file = TempFile::with_content("print \"foo\";");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut sink());
    assert!(res.is_ok());
    assert_eq!("foo\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_print_true() {
    let mut tmp_file = TempFile::with_content("print true;");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut sink());
    assert!(res.is_ok());
    assert_eq!("true\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_print_boolean_comparison() {
    // let mut tmp_file = TempFile::with_content("print false != true;");
    let mut tmp_file = TempFile::with_content("print true != false;");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut sink());
    assert!(res.is_ok());
    assert_eq!("true\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_print_multiple_statements() {
    let mut tmp_file = TempFile::with_content("print \"foo\"; print 42;");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut sink());
    assert!(res.is_ok());
    assert_eq!(
        r#"foo
42
"#,
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn run_print_multiple_lines_1() {
    let mut tmp_file = TempFile::with_content(
        r#"print "foo";
print 42;"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut sink());
    assert!(res.is_ok());
    assert_eq!(
        r#"foo
42
"#,
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn run_print_multiple_lines_2() {
    let mut tmp_file = TempFile::with_content(
        r#"
print true != true;

print "36
10
78
";

print "There should be an empty line above this.";"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut sink());
    assert!(res.is_ok());
    assert_eq!(
        r#"false
36
10
78

There should be an empty line above this.
"#,
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn run_expression_statements() {
    let mut tmp_file = TempFile::with_content(
        r#"(85 + 64 - 59) > (30 - 85) * 2;
print !false;
"world" + "hello" + "foo" + "quz" == "worldhellofooquz";
print !false;"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut sink());
    assert!(res.is_ok());
    assert_eq!(
        r#"true
true
"#,
        String::from_utf8_lossy(&output)
    );
}
