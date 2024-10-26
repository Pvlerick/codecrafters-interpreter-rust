use crate::common::interpreter;

mod common;

#[test]
fn r#if_true() {
    let (output, err) = interpreter::run_content("if (true) print 42;");
    assert_none!(err);
    assert_eq!("42\n", output);
}

#[test]
fn r#if_false() {
    let (output, err) = interpreter::run_content("if (false) print 42;");
    assert_none!(err);
    assert_eq!("", output);
}

#[test]
fn r#if_using_variable() {
    let (output, err) = interpreter::run_content(
        r#"var foo = true;
if (foo) print 42;"#,
    );
    assert_none!(err);
    assert_eq!("42\n", output);
}

#[test]
fn r#if_else_1() {
    let (output, err) = interpreter::run_content(
        r#"if (true) {
    print 42;
} else {
    print 84;
}"#,
    );
    assert_none!(err);
    assert_eq!("42\n", output);
}

#[test]
fn r#if_else_2() {
    let (output, err) = interpreter::run_content(
        r#"if (false) {
    print 42;
} else {
    print 84;
}"#,
    );
    assert_none!(err);
    assert_eq!("84\n", output);
}

#[test]
fn r#if_emtpy_else() {
    let (output, err) = interpreter::run_content(
        r#"if (false) {
    print 42;
} else {
}"#,
    );
    assert_none!(err);
    assert_eq!("", output);
}

#[test]
fn r#while_add_print_total() {
    let (output, err) = interpreter::run_content(
        r#"var i = 0;
while (i < 3) i = i + 1;
print i;"#,
    );
    assert_none!(err);
    assert_eq!("3\n", output);
}

#[test]
fn r#while_add_print_in_loop() {
    let (output, err) = interpreter::run_content(
        r#"var i = 0;
while (i < 3) {
    print i;
    i = i + 1;
}"#,
    );
    assert_none!(err);
    assert_eq!("0\n1\n2\n", output);
}

#[test]
fn r#for_print_in_loop() {
    let (output, err) = interpreter::run_content(r#"for (var i = 0; i < 10; i = i + 1) print i;"#);
    assert_none!(err);
    assert_eq!("0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n", output);
}

#[test]
fn r#for_variable_init_outside() {
    let (output, err) = interpreter::run_content(
        r#"var i = 0;
for (; i < 10; i = i + 1) print i;"#,
    );
    assert_none!(err);
    assert_eq!("0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n", output);
}
