pub mod bindings;

pub type MiniSatFFI = bindings::minisat;
pub type MiniSatPtr = *mut bindings::minisat_solver;

impl MiniSatFFI {
    pub fn load(name: &str) -> Self {
        unsafe { Self::new(libloading::library_filename(name)) }
            .unwrap_or_else(|e| panic!("Could not load shared library '{}': {}", name, e))
    }

    pub fn init(&self) -> MiniSatPtr {
        unsafe { self.minisat_new() }
    }
}

macro_rules! instance {
    ($name:expr) => {{
        use ::once_cell::sync::OnceCell;
        static INSTANCE: OnceCell<MiniSatFFI> = OnceCell::new();
        INSTANCE.get_or_init(|| MiniSatFFI::load($name))
    }};
}

impl MiniSatFFI {
    pub fn instance() -> &'static Self {
        instance!("minisat-c")
    }
}

impl MiniSatFFI {
    pub fn minisat_l_true(&self) -> &'static bindings::minisat_lbool {
        use once_cell::sync::OnceCell;
        static INSTANCE: OnceCell<bindings::minisat_lbool> = OnceCell::new();
        INSTANCE.get_or_init(|| unsafe { self.minisat_get_l_True() })
    }
    pub fn minisat_l_false(&self) -> &'static bindings::minisat_lbool {
        use once_cell::sync::OnceCell;
        static INSTANCE: OnceCell<bindings::minisat_lbool> = OnceCell::new();
        INSTANCE.get_or_init(|| unsafe { self.minisat_get_l_False() })
    }
    pub fn minisat_l_undef(&self) -> &'static bindings::minisat_lbool {
        use once_cell::sync::OnceCell;
        static INSTANCE: OnceCell<bindings::minisat_lbool> = OnceCell::new();
        INSTANCE.get_or_init(|| unsafe { self.minisat_get_l_Undef() })
    }

    // pub fn minisat_l_true(&self) -> bindings::minisat_lbool {
    //     unsafe { self.minisat_get_l_True() }
    // }
    // pub fn minisat_l_false(&self) -> bindings::minisat_lbool {
    //     unsafe { self.minisat_get_l_False() }
    // }
    // pub fn minisat_l_undef(&self) -> bindings::minisat_lbool {
    //     unsafe { self.minisat_get_l_Undef() }
    // }
}
