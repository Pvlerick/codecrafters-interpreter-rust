use crate::common::interpreter;

mod common;

#[test]
fn print_class() {
    let (output, err) = interpreter::run_content(
        r#"class DevonshireCream {
    serveOn() {
        return "Scones";
    }
}

print DevonshireCream;"#,
    );
    assert_none!(err);
    assert_eq!("class DevonshireCream {...}\n", output);
}

#[test]
fn create_instance() {
    let (_, err) = interpreter::run_content(
        r#"class Bagel {}

var bagel = Bagel();"#,
    );
    assert_none!(err);
}

#[test]
fn get_property() {
    let (output, err) = interpreter::run_content(
        r#"class Breakfast {
}

var breakfast = Breakfast();
print breakfast.meat;"#,
    );
    assert_none!(err);
    assert_eq!("nil\n", output);
}

#[test]
fn set_get_property() {
    let (output, err) = interpreter::run_content(
        r#"class Breakfast {
}

var breakfast = Breakfast();
breakfast.meat = "sausage";
print breakfast.meat;"#,
    );
    assert_none!(err);
    assert_eq!("sausage\n", output);
}
