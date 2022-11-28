pub mod http;
pub mod messages;

#[macro_export]
macro_rules! uor_res {
    ( $e:expr, $ret:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => {
                return $ret();
            }
        }
    };
}

#[macro_export]
macro_rules! uor_opt {
    ( $e:expr, $ret:expr ) => {
        match $e {
            Some(x) => x,
            None => {
                return $ret();
            }
        }
    };
}
