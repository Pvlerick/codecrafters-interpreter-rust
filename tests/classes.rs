use crate::common::interpreter;

mod common;

#[test]
fn run_class() {
    let (output, err) = interpreter::run_content(
        r#"class DevonshireCream {
    serveOn() {
        return "Scones";
    }
}

print DevonshireCream;"#,
    );
    assert!(err.is_none());
    assert_eq!("DevonshireCream{}\n", output);
}
