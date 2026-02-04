use std::{fs, path::PathBuf};

use bankai_hints::types::SyncCommitteeUpdateProofCairo;
use cairo_runner::test_hint_processor::TestHintProcessor;
use cairo_runner::test_hints::BlockSignerFixture;
use cairo_vm_base::vm::cairo_vm::{
    cairo_run::{self, cairo_run_program_with_initial_scope},
    types::{exec_scope::ExecutionScopes, layout_name::LayoutName, program::Program},
};
use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    manifest: PathBuf,
}

#[derive(Debug, Deserialize)]
struct FixtureManifest {
    tests: Vec<FixtureTest>,
}

#[derive(Debug, Deserialize)]
struct FixtureTest {
    name: String,
    program: String,
    fixture: String,
    kind: FixtureKind,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FixtureKind {
    BlockSigner,
    CommitteeUpdate,
}

struct Failure {
    test_name: String,
    fixture_index: usize,
    error: String,
}

fn load_program(path: &str) -> Result<Program, Box<dyn std::error::Error>> {
    let program_file = fs::read(path)?;
    let cairo_run_config = cairo_run::CairoRunConfig {
        allow_missing_builtins: Some(true),
        layout: LayoutName::all_cairo,
        ..Default::default()
    };
    let program = Program::from_bytes(&program_file, Some(cairo_run_config.entrypoint))?;
    Ok(program)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let manifest_str = fs::read_to_string(args.manifest)?;
    let manifest: FixtureManifest = serde_json::from_str(&manifest_str)?;

    let mut failures: Vec<Failure> = Vec::new();

    for test in manifest.tests {
        println!("Running fixture test: {}", test.name);
        let program = load_program(&test.program)?;
        let fixture_data = fs::read_to_string(&test.fixture)?;

        match test.kind {
            FixtureKind::BlockSigner => {
                let fixtures: Vec<BlockSignerFixture> = serde_json::from_str(&fixture_data)?;
                for (idx, fixture) in fixtures.iter().cloned().enumerate() {
                    let mut exec_scopes = ExecutionScopes::new();
                    exec_scopes.insert_value("block_signer_fixtures", vec![fixture]);

                    let cairo_run_config = cairo_run::CairoRunConfig {
                        allow_missing_builtins: Some(true),
                        layout: LayoutName::all_cairo,
                        ..Default::default()
                    };
                    let mut hint_processor = TestHintProcessor::new();
                    let res = cairo_run_program_with_initial_scope(
                        &program,
                        &cairo_run_config,
                        &mut hint_processor,
                        exec_scopes,
                    );

                    match res {
                        Ok(_) => {
                            println!("  PASS {}[{}]", test.name, idx);
                        }
                        Err(err) => {
                            println!("  FAIL {}[{}]", test.name, idx);
                            failures.push(Failure {
                                test_name: test.name.clone(),
                                fixture_index: idx,
                                error: err.to_string(),
                            });
                        }
                    }
                }
            }
            FixtureKind::CommitteeUpdate => {
                let fixtures: Vec<SyncCommitteeUpdateProofCairo> =
                    serde_json::from_str(&fixture_data)?;
                for (idx, fixture) in fixtures.into_iter().enumerate() {
                    let mut exec_scopes = ExecutionScopes::new();
                    exec_scopes.insert_value("committee_update_fixtures", vec![fixture]);

                    let cairo_run_config = cairo_run::CairoRunConfig {
                        allow_missing_builtins: Some(true),
                        layout: LayoutName::all_cairo,
                        ..Default::default()
                    };
                    let mut hint_processor = TestHintProcessor::new();
                    let res = cairo_run_program_with_initial_scope(
                        &program,
                        &cairo_run_config,
                        &mut hint_processor,
                        exec_scopes,
                    );

                    match res {
                        Ok(_) => {
                            println!("  PASS {}[{}]", test.name, idx);
                        }
                        Err(err) => {
                            println!("  FAIL {}[{}]", test.name, idx);
                            failures.push(Failure {
                                test_name: test.name.clone(),
                                fixture_index: idx,
                                error: err.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    if !failures.is_empty() {
        println!("\nFailures:");
        for failure in &failures {
            println!(
                "  {}[{}]: {}",
                failure.test_name, failure.fixture_index, failure.error
            );
        }
        std::process::exit(1);
    }

    println!("\nAll fixture tests passed.");
    Ok(())
}
