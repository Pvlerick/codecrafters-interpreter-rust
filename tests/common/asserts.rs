#[macro_export]
macro_rules! assert_none {
    ( $option:expr ) => {
        if $option.is_some() {
            dbg!(&$option);
        }
    };
}

#[macro_export]
macro_rules! assert_some {
    ( $option:expr ) => {
        if $option.is_none() {
            dbg!(&$option);
        }
    };
}
