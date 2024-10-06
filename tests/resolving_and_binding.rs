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
    assert!(err.is_none());
    assert_eq!("global\nglobal\n", output);
}
