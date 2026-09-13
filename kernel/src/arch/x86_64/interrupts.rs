static mut INIT: bool = false;
pub fn init() { unsafe { INIT = true; } }
pub fn is_init() -> bool { unsafe { INIT } }
pub fn enable() {} pub fn disable() {}
