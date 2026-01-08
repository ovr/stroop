//! Stroop CLI - Command-line interface for executing Stroop Assembly Text files.

use std::env;
use std::fs;
use std::process;
use std::time::Instant;

use stroop_assembly_text::{compile_module, parse_module};
use stroop_vm::BytecodeVm;
use stroop_vm_bytecode::{CompiledModule, FuncType, ValueType};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: stroop run [--bench] [--dump] <filename.sat>");
        process::exit(1);
    }

    match args[1].as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Usage: stroop run [--bench] [--dump] <filename.sat>");
                process::exit(1);
            }

            let mut bench = false;
            let mut dump = false;
            let mut filename_idx = 2;

            for i in 2..args.len() {
                match args[i].as_str() {
                    "--bench" => bench = true,
                    "--dump" => dump = true,
                    _ if !args[i].starts_with("--") => {
                        filename_idx = i;
                        break;
                    }
                    _ => {
                        eprintln!("Unknown flag: {}", args[i]);
                        process::exit(1);
                    }
                }
            }

            if filename_idx >= args.len() {
                eprintln!("Usage: stroop run [--bench] [--dump] <filename.sat>");
                process::exit(1);
            }

            run_file(&args[filename_idx], bench, dump);
        }
        "dump" => {
            if args.len() < 3 {
                eprintln!("Usage: stroop dump <filename.sat>");
                process::exit(1);
            }
            dump_bytecode(&args[2]);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Commands:");
            eprintln!("  run [--bench] [--dump] <file>  Run a .sat file");
            eprintln!("  dump <file>                    Dump bytecode instructions");
            process::exit(1);
        }
    }
}

fn dump_bytecode(filename: &str) {
    let source = fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", filename, e);
        process::exit(1);
    });

    let module = parse_module(&source).unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        process::exit(1);
    });

    let compiled = compile_module(&module).unwrap_or_else(|e| {
        eprintln!("Compile error: {}", e);
        process::exit(1);
    });
    print_compiled_module(&compiled);
}

fn print_compiled_module(compiled: &CompiledModule) {
    // Show types if non-empty
    if !compiled.types.is_empty() {
        println!("; {} types", compiled.types.len());
        for (i, t) in compiled.types.iter().enumerate() {
            println!("{:4}: {}", i, t);
        }
        println!();
    }

    // Show imports if non-empty
    if !compiled.imports.is_empty() {
        println!("; {} imports", compiled.imports.len());
        for (i, import) in compiled.imports.iter().enumerate() {
            println!(
                "{:4}: {}::{} {}",
                i, import.module, import.name, import.func_type
            );
        }
        println!();
    }

    // Show functions if non-empty
    if !compiled.functions.is_empty() {
        println!("; {} functions", compiled.functions.len());
        for (i, func) in compiled.functions.iter().enumerate() {
            let name = func.name.as_deref().unwrap_or("<anon>");
            let locals_str = if func.locals.is_empty() {
                String::new()
            } else {
                format!(
                    " [locals: {}]",
                    func.locals
                        .iter()
                        .map(|v| format!("{}", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            println!("{:4}: {}{} {}", i, name, locals_str, func.func_type);
        }
        println!();
    }

    // Show constant pool if non-empty
    if !compiled.constant_pool.is_empty() {
        println!("; constant pool ({} entries)", compiled.constant_pool.len());
        for (i, constant) in compiled.constant_pool.iter().enumerate() {
            println!("{:4}: {:?}", i, constant);
        }
        println!();
    }

    println!("; {} instructions", compiled.instructions.len());
    for (i, instr) in compiled.instructions.iter().enumerate() {
        println!("{:4}: {:?}", i, instr);
    }
}

fn run_file(filename: &str, bench: bool, dump: bool) {
    let source = fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", filename, e);
        process::exit(1);
    });

    let module = parse_module(&source).unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        process::exit(1);
    });

    let compiled = compile_module(&module).unwrap_or_else(|e| {
        eprintln!("Compile error: {}", e);
        process::exit(1);
    });

    if dump {
        print_compiled_module(&compiled);
    }

    let mut vm = BytecodeVm::new();

    vm.register_host_fn(
        "console",
        "log",
        FuncType::with_params_results(vec![ValueType::F64], vec![ValueType::F64]),
        |args| {
            println!("{}", format_f64_js(args[0].as_f64()));
            Ok(Some(args[0]))
        },
    );

    let start = Instant::now();
    match vm.execute(&compiled) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            process::exit(1);
        }
    }

    if bench {
        let elapsed = start.elapsed();
        eprintln!("Elapsed: {:.3?}", elapsed);
    }
}

/// Format f64 like Node.js console.log does
fn format_f64_js(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    if v == 0.0 {
        return "0".to_string();
    }

    let abs = v.abs();
    if abs >= 1e21 || abs < 1e-6 {
        let s = format!("{:.*e}", 16, v);
        let s = trim_mantissa_zeros(&s);
        if let Some(pos) = s.find('e') {
            let (mantissa, exp) = s.split_at(pos);
            let exp_part = &exp[1..];
            if !exp_part.starts_with('-') {
                return format!("{}e+{}", mantissa, exp_part);
            }
        }
        s
    } else {
        format!("{}", v)
    }
}

fn trim_mantissa_zeros(s: &str) -> String {
    if let Some(e_pos) = s.find('e') {
        let (mantissa, exp) = s.split_at(e_pos);
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        format!("{}{}", mantissa, exp)
    } else {
        s.to_string()
    }
}
