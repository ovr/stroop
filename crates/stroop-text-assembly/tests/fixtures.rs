use stroop_text_assembly::{compile_module, parse_module};

type TestResult = Result<(), Box<dyn std::error::Error>>;

macro_rules! fixture_test {
    ($name:ident, $file:expr) => {
        mod $name {
            use super::*;

            const SOURCE: &str = include_str!(concat!("../../../examples/", $file));

            #[test]
            fn parse_ast() -> TestResult {
                let module = parse_module(SOURCE)?;
                insta::assert_debug_snapshot!(module);
                Ok(())
            }

            #[test]
            fn compile_bytecode() -> TestResult {
                let module = parse_module(SOURCE)?;
                let compiled = compile_module(&module)?;
                insta::assert_debug_snapshot!(compiled);
                Ok(())
            }
        }
    };
}

fixture_test!(hello, "hello.sat");
fixture_test!(bench, "bench.sat");
