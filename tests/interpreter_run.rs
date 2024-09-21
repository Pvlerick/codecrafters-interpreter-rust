use std::io::stderr;

use interpreter_starter_rust::interpreter::Interpreter;

use crate::common::TempFile;

mod common;

#[test]
fn run_print_string() {
    let mut tmp_file = TempFile::with_content("print \"foo\";");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!("foo\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_print_true() {
    let mut tmp_file = TempFile::with_content("print true;");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!("true\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_print_boolean_comparison() {
    // let mut tmp_file = TempFile::with_content("print false != true;");
    let mut tmp_file = TempFile::with_content("print true != false;");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!("true\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_print_multiple_statements() {
    let mut tmp_file = TempFile::with_content("print \"foo\"; print 42;");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
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
    let res = interpreter.run(&mut output, &mut stderr());
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
    let res = interpreter.run(&mut output, &mut stderr());
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
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!(
        r#"true
true
"#,
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn run_var_print_1() {
    let mut tmp_file = TempFile::with_content(
        r#"var a = "foo";
print a;"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!("foo\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_var_print_2() {
    let mut tmp_file = TempFile::with_content(
        r#"var a = 20 + 22;
print a;"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!("42\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_var_redeclare_print_2() {
    let mut tmp_file = TempFile::with_content(
        r#"var a = 20 + 22;
var a = "foo";
print a;"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!("foo\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_var_unassigned() {
    let mut tmp_file = TempFile::with_content(
        r#"var a;
print a;"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!("nil\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_var_undefined() {
    let mut tmp_file = TempFile::with_content("print a;");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_err());
}

#[test]
fn run_var_assignment_1() {
    let mut tmp_file = TempFile::with_content(
        r#"var a = 20;
a = 42;
print a;"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!("42\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_var_assignment_2() {
    let mut tmp_file = TempFile::with_content(
        r#"var a = 20;
a = a + 22;
print a;"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!("42\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_var_assignment_3() {
    let mut tmp_file = TempFile::with_content(
        r#"var quz;
quz = 1;
print quz;
print quz = 2;
print quz;
"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!(
        r#"1
2
2
"#,
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn run_blocks_1() {
    let mut tmp_file = TempFile::with_content(
        r#"{
    var hello = "baz";
    print hello;
}"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!("baz\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_blocks_2() {
    let mut tmp_file = TempFile::with_content(
        r#"{
    var world = "before";
    print world;
}
{
    var world = "after";
    print world;
}"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!(
        r#"before
after
"#,
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn run_blocks_3() {
    let mut tmp_file = TempFile::with_content(
        r#"{
    var hello = 88;
    {
        var foo = hello;
        print foo;
    }
    print hello;
}"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!(
        r#"88
88
"#,
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn run_blocks_4() {
    let mut tmp_file = TempFile::with_content(
        r#"{
    var foo = 88;
    {
        var foo = 42;
        print foo;
    }
    print foo;
}"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_ok());
    assert_eq!(
        r#"42
88
"#,
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn run_blocks_5() {
    let mut tmp_file = TempFile::with_content(
        r#"{
    var foo = 11;
}
print foo"#,
    );
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut stderr());
    assert!(res.is_err());
}
