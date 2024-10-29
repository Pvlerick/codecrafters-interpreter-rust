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
fn this() {
    let (output, err) = interpreter::run_content(
        r#"class Person {
    sayName() {
        print this.name;
    }
}

var jane = Person();
jane.name = "Jane";

jane.sayName();"#,
    );
    assert_none!(err);
    assert_eq!("Jane\n", output);
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
    assert_eq!("Jane\n", output);
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
    assert_eq!("Jane\n", output);
}

#[test]
fn invalid_use_of_this_1() {
    let (_, err) = interpreter::run_content("print this;");
    assert_some!(err);
}

#[test]
fn invalid_use_of_this_2() {
    let (_, err) = interpreter::run_content(
        r#"fun notAMethod() {
print this;
}"#,
    );
    assert_some!(err);
}

#[test]
fn init() {
    let (output, err) = interpreter::run_content(
        r#"class Person {
    init(firstName) {
        this.firstName = firstName;
        this.lastName = "Doe";
    }
}

var p = Person("Jane");
print p.firstName + " " + p.lastName;"#,
    );
    assert_none!(err);
    assert_eq!("Jane Doe\n", output);
}

#[test]
fn init_called_directly() {
    let (output, err) = interpreter::run_content(
        r#"class Foo {
    init() {
        print this;
    }
}

var foo = Foo();
print foo.init();"#,
    );
    assert_none!(err);
    assert_eq!("Foo instance\nFoo instance\nnil\n", output);
}

#[test]
fn init_return_1() {
    let (_, err) = interpreter::run_content(
        r#"class Foo {
    init() {
        return;
    }
}

var foo = Foo();"#,
    );
    assert_none!(err);
}

#[test]
fn init_return_2() {
    let (_, err) = interpreter::run_content(
        r#"class Foo {
    init() {
        return "hello";
    }
}

var foo = Foo();"#,
    );
    assert_some!(err);
}

#[test]
fn init_return_3() {
    let (output, err) = interpreter::run_content(
        r#"class Foo {
    init() {
        if (true) {
            return;
        }
        print "Foo";
    }

    bar() {
        print "Foo.bar";
    }
}

var foo = Foo();
foo.bar();"#,
    );
    assert_none!(err);
    assert_eq!("Foo.bar\n", output);
}
