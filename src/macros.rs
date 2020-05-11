macro_rules! wrap {
    ($val:ident, $res:ident) => (
        if $res == EXIT_SUCCESS {
            Ok($val)
        } else {
            Err($res)
        }
    )
}


