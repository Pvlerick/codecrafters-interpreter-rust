use crate::common::interpreter;

mod common;

#[test]
fn static_scope() {
    let (output, err) = interpreter::run_content(
        r#"var a = "global";
{
  fun showA() {
    print a;
  }

  showA();
  var a = "block";
  showA();
}"#,
    );
    assert_none!(err);
    assert_eq!("global\nglobal\n", output);
}

#[test]
fn variable_self_in_initializer() {
    let (_, err) = interpreter::run_content("var a = a;");
    assert_some!(err);
}

#[test]
fn return_at_top_level() {
    let (_, err) = interpreter::run_content("return \"at top level\";");
    assert_some!(err);
}

#[test]
fn multiple_variables_with_same_name_in_local_scope() {
    let (_, err) = interpreter::run_content(
        "fun bad() {
  var a = 42;
  var a = 84;
}",
    );
    assert_some!(err);
}
