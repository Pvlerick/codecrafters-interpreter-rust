use crate::common::interpreter;

mod common;

#[test]
fn function_clock() {
    let (_, err) = interpreter::run_content("print clock();");
    assert_none!(err);
}

#[test]
fn function_env() {
    std::env::set_var("LOX_TEST_ENV_VAR", "foo");
    let (output, err) = interpreter::run_content("print env(\"LOX_TEST_ENV_VAR\");");
    assert_none!(err);
    assert_eq!("foo\n", output);
}

#[test]
fn function_print_42() {
    let (output, err) = interpreter::run_content(
        r#"fun print42() {
    print 42;
}

print42();
print42();"#,
    );
    assert_none!(err);
    assert_eq!("42\n42\n", output);
}

#[test]
fn function_print_sum() {
    let (output, err) = interpreter::run_content(
        r#"fun printSum(a, b) {
    print a + b;
}

printSum(42, 5);"#,
    );
    assert_none!(err);
    assert_eq!("47\n", output);
}

#[test]
fn function_return_value() {
    let (output, err) = interpreter::run_content(
        r#"fun sum(a, b) {
    return a + b;
}

print sum(37, 4) + 1;"#,
    );
    assert_none!(err);
    assert_eq!("42\n", output);
}

#[test]
fn function_return() {
    let (output, err) = interpreter::run_content(
        r#"fun printIf(a, b) {
    if (a > b) {
        return;
    }
    print 42;
}

printIf(9, 2);"#,
    );
    assert_none!(err);
    assert_eq!("", output);
}

#[test]
fn function_nested_returns() {
    let (output, err) = interpreter::run_content(
        r#"fun f(n) {
  if (n > 0) {
    if (n > 5) {
      print "n is > 5";
      return;
    }
    print "n is <= 5 but > 0";
    return;
  }
  print "n is <= 0";
}

f(9);"#,
    );
    assert_none!(err);
    assert_eq!("n is > 5\n", output);
}

#[test]
fn function_nested_returns_in_loop() {
    let (output, err) = interpreter::run_content(
        r#"fun f(n) {
  while (n < 100) {
    if (n == 3) {
      return n;
    }
    print n;
    n = n + 1;
  }
  print "here?";
}

print f(1);"#,
    );
    assert_none!(err);
    assert_eq!("1\n2\n3\n", output);
}

#[test]
fn function_closure() {
    let (output, err) = interpreter::run_content(
        r#"fun makeCounter() {
  var i = 0;
  fun count() {
    i = i + 1;
    print i;
  }
  return count;
}

var counter = makeCounter();
counter();
counter();"#,
    );
    assert_none!(err);
    assert_eq!("1\n2\n", output);
}

#[test]
fn function_anonymous() {
    let (output, err) = interpreter::run_content(
        r#"fun c(a, b) {
  a(b);
}

c(fun(a) {
  print a;
}, "foo");"#,
    );
    assert_none!(err);
    assert_eq!("foo\n", output);
}

#[test]
fn function_expression() {
    let (output, err) = interpreter::run_content("fun () {};");
    assert_none!(err);
    assert_eq!("", output);
}

#[test]
fn function_err_not_a_function() {
    let (_, err) = interpreter::run_content(r#""not a function();"#);
    assert_some!(err);
}
