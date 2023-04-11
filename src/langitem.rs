#[lang = "eh_personality"]
extern "C" fn rust_eh_personality() {}

#[lang = "panic_impl"]
extern "C" fn rust_begin_panic(_: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort()
}
