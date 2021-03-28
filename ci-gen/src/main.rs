use gh_actions_gen::actions::cargo_cache;
use gh_actions_gen::actions::cargo_doc;
use gh_actions_gen::actions::cargo_test;
use gh_actions_gen::actions::checkout_sources;
use gh_actions_gen::actions::rust_install_toolchain;
use gh_actions_gen::actions::RustToolchain;
use gh_actions_gen::ghwf::Env;
use gh_actions_gen::ghwf::Job;
use gh_actions_gen::ghwf::Step;
use gh_actions_gen::rustfmt::rustfmt_check_job;
use gh_actions_gen::super_linter::super_linter_job;

fn steps(os: Os, channel: RustToolchain) -> Vec<Step> {
    let mut steps = Vec::new();
    steps.push(checkout_sources());
    steps.push(rust_install_toolchain(channel));
    steps.push(install_protobuf_step());
    steps.push(Step::run("run", "ci/run.sh"));
    steps
}

#[derive(PartialEq, Eq, Copy, Clone)]
struct Os {
    name: &'static str,
    ghwf: Env,
}

const LINUX: Os = Os {
    name: "linux",
    ghwf: Env::UbuntuLatest,
};
const MACOS: Os = Os {
    name: "macos",
    ghwf: Env::MacosLatest,
};
const _WINDOWS: Os = Os {
    name: "windows",
    ghwf: Env::WindowsLatest,
};

fn install_protobuf_step() -> Step {
    Step::run("install protobuf", "./ci/install-protobuf.sh")
}

fn protobuf_version_env() -> (String, String) {
    ("PROTOBUF_VERSION".to_owned(), "3.1.0".to_owned())
}

fn rust_backtrace_env() -> (String, String) {
    ("RUST_BACKTRACE".to_owned(), "1".to_owned())
}

fn cargo_doc_job() -> Job {
    let os = LINUX;
    let mut steps = Vec::new();
    steps.push(checkout_sources());
    steps.push(install_protobuf_step());
    steps.push(rust_install_toolchain(RustToolchain::Stable));
    steps.push(cargo_doc("cargo doc", ""));
    Job {
        id: "cargo-doc".to_owned(),
        name: "cargo doc".to_owned(),
        runs_on: os.ghwf,
        env: vec![protobuf_version_env(), rust_backtrace_env()],
        steps,
        ..Default::default()
    }
}

fn test_protoc_plugin_job() -> Job {
    let mut steps = Vec::new();
    steps.push(cargo_cache());
    steps.push(checkout_sources());
    steps.push(rust_install_toolchain(RustToolchain::Stable));
    steps.push(install_protobuf_step());
    steps.push(Step::run(
        "install protobuf-codegen",
        "cargo install protobuf-codegen --version=2.18.2",
    ));
    steps.push(Step::run("gen", "grpc-compiler/test-protoc-plugin/gen.sh"));
    steps.push(Step::run(
        "check",
        "cargo check --manifest-path grpc-compiler/test-protoc-plugin/Cargo.toml",
    ));
    Job {
        id: "test-protoc-plugin".to_owned(),
        name: "test-protoc-plugin".to_owned(),
        env: vec![protobuf_version_env(), rust_backtrace_env()],
        steps,
        ..Default::default()
    }
}

fn jobs() -> Vec<Job> {
    let mut r = Vec::new();
    for &channel in &[
        RustToolchain::Stable,
        RustToolchain::Beta,
        RustToolchain::Nightly,
    ] {
        for &os in &[LINUX, MACOS] {
            if channel == RustToolchain::Beta && os == MACOS {
                // skip some jobs because macos is expensive
                continue;
            }
            r.push(Job {
                id: format!("{}-{}", os.name, channel),
                name: format!("{} {}", os.name, channel),
                runs_on: os.ghwf,
                env: vec![protobuf_version_env(), rust_backtrace_env()],
                steps: steps(os, channel),
                ..Default::default()
            });
        }
    }

    r.push(test_protoc_plugin_job());

    r.push(cargo_doc_job());

    // TODO: enable
    //r.push(rustfmt_check_job());

    r.push(super_linter_job());

    r
}

fn main() {
    gh_actions_gen::write(jobs());
}
