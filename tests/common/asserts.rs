#[macro_export]
macro_rules! assert_none {
    ( $option:expr ) => {
        if $option.is_some() {
            dbg!(&$option);
            panic!("{} is some", stringify!($option));
        }
    };
}

#[macro_export]
macro_rules! assert_some {
    ( $option:expr ) => {
        if $option.is_none() {
            panic!("{} is none", stringify!($option));
        }
    };
}
