#![feature(lang_items, core_intrinsics)]
#![no_std]
#![no_main]
#![cfg_attr(target_os = "windows", windows_subsystem = "console")]

use core::ffi::{c_char, CStr};
use derivative::Derivative;
use fchashmap::FcHashMap;
use libc_print::libc_println;
use static_alloc::Bump;
use without_alloc::alloc::LocalAllocLeakExt;
use without_alloc::{Box, FixedVec};

static SLAB: Bump<[Json<'static>; 4096]> = Bump::uninit();

fn print_array<'a>(
    arr: &Option<FixedVec<'a, Json<'a>>>,
    fmt: &mut core::fmt::Formatter,
) -> core::fmt::Result {
    write!(fmt, "{:#?}", arr.as_ref().map(|x| x.as_slice()))
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum Json<'a> {
    Object(FcHashMap<&'a str, Option<Box<'a, Json<'a>>>, 64>),
    Array(#[derivative(Debug(format_with = "print_array"))] Option<FixedVec<'a, Json<'a>>>),
    Null,
    Number(&'a str),
    Bool(bool),
    String(&'a str),
}

peg::parser!(grammar parser() for str {
    // JSON grammar (RFC 4627). Note that this only checks for valid JSON and does not build a syntax
    // tree.

    pub rule json() -> Json<'input> = _ s:(value()) _ { s }

    rule _() = [' ' | '\t' | '\r' | '\n']*
    rule value_separator() = _ "," _

    rule value() -> Json<'input>
        =
         b:("true" {true} / "false" {false}) { Json::Bool(b) }
        / "null" { Json::Null }
        / s:$("-"? int() frac()? exp()?) { Json::Number(s) }
        / s:string() { Json::String(s) }
        / "{" _ pair:(key:string() _ ":" _ value:value() { (key, SLAB.boxed(value)) }) **<, 1024> value_separator() _ "}"  {
            Json::Object(pair.into_iter().collect())
        }
        / "[" _ val:value() **<, 128> value_separator() _ "]" {
            let mut vec = SLAB.fixed_vec(val.len());
            if let Some(ref mut vec) = vec {
                vec.extend(val.into_iter());
            }
            Json::Array(vec)
        }


    rule int()
        = ['0'] / ['1'..='9']['0'..='9']*

    rule exp()
        = ("e" / "E") ("-" / "+")? ['0'..='9']*<1,32>

    rule frac()
        = "." ['0'..='9']*<1,32>

    // note: escaped chars not handled
    rule string() -> &'input str = str:$("\"" (!"\"" [_])* "\"") {
        let mut s = str.chars();
        s.next();
        s.next_back();
        s.as_str()
    }
});

#[no_mangle]
pub extern "C" fn main(argc: usize, argv: *const *const c_char) -> isize {
    let args = unsafe { core::slice::from_raw_parts(argv, argc) };
    let mut args = args.into_iter().map(|&x| unsafe { CStr::from_ptr(x) });

    match args.nth(1) {
        Some(arg) => libc_println!("{:#?}", parser::json(arg.to_str().unwrap())),
        None => {}
    };

    0
}

mod langitem;

mod windows_shim;
