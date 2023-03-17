pub mod lin_alg;

#[macro_export]
macro_rules! offset_of {
    ($type:ty, $field:ident) => {{
        unsafe { &(*(0 as usize as *const $type)).$field as *const _ as usize }
    }};
}
