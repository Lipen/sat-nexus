pub mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clippy::style)]

    include!(concat!(env!("OUT_DIR"), "/bindings-cminisat-static.rs"));
    // include!("../_bindings-cminisat-static.rs");
}

#[cfg(test)]
mod tests {
    use super::*;
    use bindings::*;

    #[test]
    fn test_lbool() {
        unsafe {
            let lbool_true = minisat_l_True;
            println!("minisat_l_True = {:?}", lbool_true);
            assert_eq!(lbool_true, 1);

            let lbool_false = minisat_l_False;
            println!("minisat_l_False = {:?}", lbool_false);
            assert_eq!(lbool_false, 0);

            let lbool_undef = minisat_l_Undef;
            println!("minisat_l_Undef = {:?}", lbool_undef);
            assert_eq!(lbool_undef, -1);
        }
    }
}
