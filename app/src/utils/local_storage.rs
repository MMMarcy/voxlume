use web_sys;

pub(crate) fn local_storage() -> Option<web_sys::Storage> {
    #[cfg(feature = "hydrate")]
    {
        use leptos::prelude::*;
        return window().local_storage().unwrap();
    }
    #[allow(unreachable_code)]
    #[cfg(feature = "ssr")]
    {
        None // Return None on the server
    }
}
