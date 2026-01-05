use stroop_text_assembly::{compile_module, parse_module};

macro_rules! fixture_test {
    ($name:ident, $file:expr) => {
        mod $name {
            use super::*;

            const SOURCE: &str = include_str!(concat!("../../../examples/", $file));

            #[test]
            fn parse_ast() {
                let module = parse_module(SOURCE).unwrap();
                insta::assert_debug_snapshot!(module);
            }

            #[test]
            fn compile_bytecode() {
                let module = parse_module(SOURCE).unwrap();
                let compiled = compile_module(&module);
                insta::assert_debug_snapshot!(compiled);
            }
        }
    };
}

fixture_test!(hello, "hello.sat");
fixture_test!(bench, "bench.sat");
