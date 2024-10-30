use crate::common::interpreter;

mod common;

#[test]
fn print_class() {
    let (output, err) = interpreter::run_content(
        r#"class Doughnut {
    tastes() {
        return "good";
    }
}

class BostonCream < Doughnut {}

var bc = BostonCream();
print bc.tastes();"#,
    );
    assert_none!(err);
    assert_eq!("good\n", output);
}

#[test]
fn cant_inherit_from_non_existing() {
    let (_, err) = interpreter::run_content("class A < B {}");
    assert_some!(err);
}

#[test]
fn cant_inherit_from_self() {
    let (_, err) = interpreter::run_content("class A < A {}");
    assert_some!(err);
}

#[test]
fn must_inherit_from_class() {
    let (_, err) = interpreter::run_content(
        r#"var NotAClass = "I'm not a class";
        class A < NotAClass {}"#,
    );
    assert_some!(err);
}

#[test]
fn two_levels_of_inheritance() {
    let (output, err) = interpreter::run_content(
        r#"class Doughnut {
    tastes() {
        return "good";
    }
}

class BostonCream < Doughnut {}

class WhippedCreameToppedBoston < BostonCream {}

var wctb = WhippedCreameToppedBoston();
print wctb.tastes();"#,
    );
    assert_none!(err);
    assert_eq!("good\n", output);
}

#[test]
fn super_call() {
    let (output, err) = interpreter::run_content(
        r#"class Doughnut {
    tastes() {
        print "good";
    }
}

class BostonCream < Doughnut {
    tastes() {
        super.tastes();
        print "no wait, very good!";
    }
}

var bc = BostonCream();
bc.tastes();"#,
    );
    assert_none!(err);
    assert_eq!("good\nno wait, very good!\n", output);
}

#[test]
fn super_store_method_in_variable() {
    let (output, err) = interpreter::run_content(
        r#"class Doughnut {
    tastes() {
        print "good";
    }
}

class BostonCream < Doughnut {
    tastes() {
        var st = super.tastes;
        st();
        print "no wait, very good!";
    }
}

var bc = BostonCream();
bc.tastes();"#,
    );
    assert_none!(err);
    assert_eq!("good\nno wait, very good!\n", output);
}

#[test]
fn super_binding() {
    let (output, err) = interpreter::run_content(
        r#"class A {
    method() {
        print "A method";
    }
}

class B < A {
    method() {
        print "B method";
    }

    test() {
        super.method();
    }
}

class C < B {}

C().test();"#,
    );
    assert_none!(err);
    assert_eq!("A method\n", output);
}

#[test]
fn super_outside_class() {
    let (_, err) = interpreter::run_content(
        r#"fun foo() {
    super.bar();
}

foo();"#,
    );
    assert_some!(err);
}

#[test]
fn super_in_class_without_superclass() {
    let (_, err) = interpreter::run_content(
        r#"class C {
    bar() {
        super.foo();
    }
}

C().bar();"#,
    );
    assert_some!(err);
}
