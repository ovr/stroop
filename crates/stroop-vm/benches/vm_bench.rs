use codspeed_criterion_compat::{Criterion, SamplingMode, criterion_group, criterion_main};
use std::time::Duration;
use stroop_text_assembly::{FuncType, ValueType, compile_module, parse_module};
use stroop_vm::BytecodeVm;

fn bench_factorial(c: &mut Criterion) {
    let source = include_str!("../../../examples/bench.sat");
    let module = parse_module(source).unwrap();
    let compiled = compile_module(&module);

    // Use BENCH_SUFFIX env var to differentiate benchmarks across platforms in CI
    let suffix = std::env::var("BENCH_SUFFIX").unwrap_or_default();
    let group_name = if suffix.is_empty() {
        "vm".to_string()
    } else {
        format!("vm-{suffix}")
    };
    let mut group = c.benchmark_group(&group_name);
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("factorial_4m", |b| {
        b.iter(|| {
            let mut vm = BytecodeVm::new();
            vm.register_host_fn(
                "console",
                "log",
                FuncType::with_params_results(vec![ValueType::F64], vec![ValueType::F64]),
                |args| Ok(Some(args[0])),
            );
            vm.execute(&compiled).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_factorial);
criterion_main!(benches);
