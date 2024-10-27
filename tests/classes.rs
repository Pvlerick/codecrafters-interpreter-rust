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

#[test]
fn instance_as_field() {
    let (output, err) = interpreter::run_content(
        r#"class Foo {}
class Bar {}

var foo = Foo();
foo.bar = Bar();
foo.bar.baz = "hello";

print foo.bar.baz;"#,
    );
    assert_none!(err);
    assert_eq!("hello\n", output);
}

#[test]
fn fun_as_field() {
    let (output, err) = interpreter::run_content(
        r#"fun sayBar() {
    print "bar";
}

class Foo {}

var foo = Foo();
foo.sayBar = sayBar; 
foo.sayBar();"#,
    );
    assert_none!(err);
    assert_eq!("bar\n", output);
}

#[test]
fn anonymous_fun_as_field() {
    let (output, err) = interpreter::run_content(
        r#"class Foo {}

var foo = Foo();
foo.sayBar = fun() {
    print "bar";
};

foo.sayBar();"#,
    );
    assert_none!(err);
    assert_eq!("bar\n", output);
}

#[test]
fn method() {
    let (output, err) = interpreter::run_content(
        r#"class Foo {
    greet(name) {
        print "hello " + name;
    }
}

var foo = Foo();
foo.greet("bar");"#,
    );
    assert_none!(err);
    assert_eq!("hello bar\n", output);
}

#[test]
fn field_overrides_method() {
    let (output, err) = interpreter::run_content(
        r#"class Foo {
    greet(name) {
        print "hello " + name;
    }
}

var foo = Foo();
foo.greet = fun(name) {
    print name + ", you have been hijacked!";
};
foo.greet("bar");"#,
    );
    assert_none!(err);
    assert_eq!("bar, you have been hijacked!\n", output);
}

#[test]
fn method_as_variable() {
    let (output, err) = interpreter::run_content(
        r#"class Foo {
    greet(name) {
        print "hello " + name;
    }
}

var foo = Foo();
var greet = foo.greet;

greet("bar");"#,
    );
    assert_none!(err);
    assert_eq!("hello bar\n", output);
}

#[test]
fn method_as_variable_remembers_this_1() {
    let (output, err) = interpreter::run_content(
        r#"class Person {
    sayName() {
        print this.name;
    }
}

var jane = Person();
jane.name = "Jane";

var method = jane.sayName;
method();"#,
    );
    assert_none!(err);
    assert_eq!("Janex\n", output);
}

#[test]
fn method_as_variable_remembers_this_2() {
    let (output, err) = interpreter::run_content(
        r#"class Person {
    sayName() {
        print this.name;
    }
}

var jane = Person();
jane.name = "Jane";

var bill = Person();
bill.name = "Bill";

bill.sayName = jane.sayName;
bill.sayName();"#,
    );
    assert_none!(err);
    assert_eq!("Janex\n", output);
}
