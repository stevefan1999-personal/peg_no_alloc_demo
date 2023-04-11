#![feature(lang_items, start, core_intrinsics, libc)]
#![no_std]
#![no_main]
#![cfg_attr(target_os = "windows", windows_subsystem = "console")]

use arrayvec::ArrayVec;
use fchashmap::FcHashMap;
use libc_print::libc_println;
use static_alloc::Bump;

use without_alloc::{alloc::LocalAllocLeakExt, Box};

static SLAB: Bump<[Json<'static>; 1024]> = Bump::uninit();

#[derive(Debug)]
pub enum Json<'a> {
    Null,
    Number(&'a str),
    Bool(bool),
    Object(FcHashMap<&'a str, Option<Box<'a, Json<'a>>>, 24>),
    Pair {
        key: &'a str,
        value: Option<Box<'a, Json<'a>>>,
    },
    Array(ArrayVec<Option<Box<'a, Json<'a>>>, 24>),
    String(&'a str),
}

peg::parser!(grammar parser() for str {
    // JSON grammar (RFC 4627). Note that this only checks for valid JSON and does not build a syntax
    // tree.

    pub rule json() -> Json<'input> = _ s:(value()) _ { s }

    rule _() = [' ' | '\t' | '\r' | '\n']*
    rule value_separator() = _ "," _

    rule value() -> Json<'input>
        = "false" { Json::Bool(false) }
        / "true" { Json::Bool(true) }
        / "null" { Json::Null }
        / "{" _ pair:(key:string() _ ":" _ value:value() { (key, value) }) **<,32> value_separator() _ "}"  { Json::Object(pair.into_iter().map(|(k,v)| (k, SLAB.boxed(v))).collect()) }
        / "[" _ val:value() **<,32> value_separator() _ "]" { Json::Array(val.into_iter().map(|x| SLAB.boxed(x)).collect()) }
        / s:number() { Json::Number(s) }
        / s:string() { Json::String(s) }


    rule number() ->  &'input str
        = s:$("-"? int() frac()? exp()?) { s }

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
