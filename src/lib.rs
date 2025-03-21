#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::identity_op)]
#![allow(non_camel_case_types)]
#![allow(clippy::collapsible_else_if)]

pub(crate) mod cpu;
pub(crate) mod gpu;
pub(crate) mod memory_bus;
pub(crate) mod motherboard;

#[macro_export]
macro_rules! bit {
    ($val:expr, $ith:expr) => {
        (($val >> $ith) & 1 == 1)
    };
}
