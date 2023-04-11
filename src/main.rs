#![feature(lang_items, start, core_intrinsics, libc)]
#![no_std]
#![no_main]
#![cfg_attr(target_os = "windows", windows_subsystem = "console")]

#[global_allocator]
static ALLOCATOR: libc_alloc::LibcAlloc = libc_alloc::LibcAlloc;

use alloc::boxed::Box;
use libc_print::libc_println;
use peg::__private::ArrayVec;

extern crate alloc;

#[derive(Debug)]
pub enum Json<'a> {
    Null,
    Number(f64),
    Bool(bool),
    Object(ArrayVec<Box<Json<'a>>, 32>),
    Pair {
        key: Box<Json<'a>>,
        value: Box<Json<'a>>,
    },
    Array(ArrayVec<Box<Json<'a>>, 32>),
    String(&'a str),
}

peg::parser!(grammar parser() for str {
    // JSON grammar (RFC 4627). Note that this only checks for valid JSON and does not build a syntax
    // tree.

    pub rule json() -> Json<'input> = _ s:(object() / array()) _ { s }

    rule _() = [' ' | '\t' | '\r' | '\n']*
    rule value_separator() = _ "," _

    rule value() -> Json<'input>
        = "false" { Json::Bool(false) }
        / "true" { Json::Bool(true) }
        / "null" { Json::Null }
        / object()
        / array()
        / number()
        / string()

    rule object() -> Json<'input>
        = "{" _ pair:member() **<,32> value_separator() _ "}" { Json::Object(pair.into_iter().map(Box::new).collect()) }

    rule member() -> Json<'input>
        = key:string() _ ":" _ value:value() { Json::Pair { key: Box::new(key), value: Box::new(value) } }

    rule array() -> Json<'input>
        = "[" _ val:value() **<,32> value_separator() _ "]" { Json::Array(val.into_iter().map(Box::new).collect()) }

    rule number() -> Json<'input>
        = s:$("-"? int() frac()? exp()?) {? Ok(Json::Number(fast_float::parse(s).map_err(|x| "not a number")?)) }

    rule int()
        = ['0'] / ['1'..='9']['0'..='9']*

    rule exp()
        = ("e" / "E") ("-" / "+")? ['0'..='9']*<1,32>

    rule frac()
        = "." ['0'..='9']*<1,32>

    // note: escaped chars not handled
    rule string() -> Json<'input>
        = str:$("\"" (!"\"" [_])* "\"") { Json::String(str) }
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

    libc_println!("{:#?}", x);
    0
}

mod langitem;

mod windows_shim;
