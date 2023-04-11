#![feature(lang_items)]
#![feature(start)]
#![feature(core_intrinsics)]
#![feature(libc)]
#![no_std]
#![no_main]

#![cfg(target_os = "windows")]
#![windows_subsystem = "console"]

#[cfg_attr(target_os = "windows", link(name = "msvcrt"))]
extern {}


use libc_print::libc_println;

#[lang = "eh_personality"]
extern "C" fn rust_eh_personality() {}
#[lang = "panic_impl"]
extern "C" fn rust_begin_panic(_: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort()
}

peg::parser!(grammar parser() for str {
    // JSON grammar (RFC 4627). Note that this only checks for valid JSON and does not build a syntax
    // tree.

    pub rule json() = _ (object() / array()) _

    rule _() = [' ' | '\t' | '\r' | '\n']*
    rule value_separator() = _ "," _

    rule value()
        = "false" / "true" / "null" / object() / array() / number() / string()

    rule object()
        = "{" _ member() **<,128> value_separator() _ "}"

    rule member()
        = string() _ ":" _ value()

    rule array()
        = "[" _ (value() **<,128> value_separator()) _ "]"

    rule number()
        = "-"? int() frac()? exp()? {}

    rule int()
        = ['0'] / ['1'..='9']['0'..='9']*

    rule exp()
        = ("e" / "E") ("-" / "+")? ['0'..='9']*<1,32>

    rule frac()
        = "." ['0'..='9']*<1,32>

    // note: escaped chars not handled
    rule string()
        = "\"" (!"\"" [_])* "\""
});

#[no_mangle]
pub extern "C" fn main(argc: usize, argv: *const *const u8) -> isize {
    let _args = unsafe { core::slice::from_raw_parts(argv, argc) };

    let input = r#"
{
	"X": 0.6e2,
	"Y": 5,
	"Z": -5.312344,
	"Bool": false,
	"Bool": true,
	"Null": null,
	"Attr": {
		"Name": "bla",
		"Siblings": [6, 1, 2, {}, {}, {}]
	},
	"Nested Array": [[[[[[[[[]]]]]]]]],
	"Obj": {
		"Child": {
			"A": [],
			"Child": {
				"Child": {}
			}
		}
	}
}
"#;
    let x = parser::json(input);

    libc_println!("{:?}", x);
    0
}
