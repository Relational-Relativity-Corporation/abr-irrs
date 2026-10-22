/// irrs-eval binary.
///
/// Domain-independent pre-build declaration evaluator.
/// Reads a declaration from a JSON file; writes the evaluation record to a JSON file.
///
/// Usage:
///   irrs-eval evaluate --declaration <path> [--output <path>] [--auth <path>]
///
/// Flags:
///   --declaration <path>  Required. Path to the declaration JSON file.
///   --output <path>       Optional. Path to write the evaluation record.
///                         Defaults to stdout.
///   --auth <path>         Optional. Path to the authorization JSON file.
///                         Defaults to "eval/authorization.json".
///
/// Exit codes (consumed by CI):
///   0  — Authorized. Build may proceed.
///   1  — Evaluation failed or flag error. A prerequisite was not satisfied.
///   2  — Passed; awaiting Origin authorization.
///        CI uploads the record as an artifact. Build is held.

mod declaration;
mod provenance;
mod evidence;
mod authorization;
mod record;
mod evaluator;

use declaration::Declaration;
use evaluator::Evaluator;
use record::EvaluationStatus;

use sha2::{Sha256, Digest as Sha2Digest};

fn main() {
    let args = parse_args();

    // Load the declaration from the JSON file.
    let decl_text = match std::fs::read_to_string(&args.declaration_path) {
        Ok(t)  => t,
        Err(e) => {
            eprintln!("ERROR: could not read declaration file '{}': {}", args.declaration_path, e);
            std::process::exit(1);
        }
    };

    // Compute the SHA-256 digest of the raw declaration JSON text BEFORE parsing.
    // This digest is stored in the evaluation record and must match the authorization's
    // declaration_digest exactly. Any byte change to the declaration invalidates
    // an existing authorization.
    let declaration_digest = hex::encode(Sha256::digest(decl_text.as_bytes()));

    let decl: Declaration = match serde_json::from_str(&decl_text) {
        Ok(d)  => d,
        Err(e) => {
            eprintln!("ERROR: could not parse declaration file '{}': {}", args.declaration_path, e);
            eprintln!("       {}", e);
            std::process::exit(1);
        }
    };

    // Evaluate the declaration.
    let mut record = Evaluator::evaluate(decl, declaration_digest);

    // Attempt to apply authorization.
    if let Some(auth) = authorization::load(&args.auth_path) {
        authorization::apply(&mut record, auth);
    }

    // Serialize the record.
    let json = serde_json::to_string_pretty(&record)
        .expect("record serialization failed");

    // Write output.
    match &args.output_path {
        Some(path) => {
            // Ensure parent directory exists.
            if let Some(parent) = std::path::Path::new(path).parent() {
                if !parent.as_os_str().is_empty() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        eprintln!("ERROR: could not create output directory '{}': {}", parent.display(), e);
                        std::process::exit(1);
                    }
                }
            }
            if let Err(e) = std::fs::write(path, &json) {
                eprintln!("ERROR: could not write output file '{}': {}", path, e);
                std::process::exit(1);
            }
            eprintln!("Record written to: {}", path);
        }
        None => {
            println!("{}", json);
        }
    }

    // Report and exit with CI-consumable code.
    match record.status {
        EvaluationStatus::Authorized => {
            eprintln!("✅  Declaration authorized (record_id: {}). Build may proceed.", record.record_id);
            std::process::exit(0);
        }
        EvaluationStatus::PassedAwaitingAuthorization => {
            eprintln!(
                "⏸   Evaluation passed (record_id: {}).\n\
                 \n\
                 Build is held pending Origin authorization.\n\
                 Review the evaluation record, then commit:\n\
                 \n\
                     eval/authorization.json\n\
                 \n\
                 with record_id          = \"{}\"\n\
                 and declaration_digest  = \"{}\"\n\
                 \n\
                 See eval/authorization.json.template and eval/README.md for the authorization format.\n\
                 NOTE: The declaration_digest must match the record exactly. \
                 Any change to declaration.json after evaluation invalidates the digest.",
                record.record_id, record.record_id, record.declaration_digest
            );
            std::process::exit(2);
        }
        EvaluationStatus::Failed => {
            eprintln!(
                "❌  Evaluation failed: {}",
                record.failure_reason.as_deref().unwrap_or("unknown")
            );
            std::process::exit(1);
        }
    }
}

// ── CLI argument parsing ──────────────────────────────────────────────────────

struct Args {
    declaration_path: String,
    output_path:      Option<String>,
    auth_path:        String,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] != "evaluate" {
        eprintln!("Usage: irrs-eval evaluate --declaration <path> [--output <path>] [--auth <path>]");
        std::process::exit(1);
    }

    let mut declaration_path: Option<String> = None;
    let mut output_path:      Option<String> = None;
    let mut auth_path         = "eval/authorization.json".to_string();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--declaration" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("ERROR: --declaration requires a path argument");
                    std::process::exit(1);
                }
                declaration_path = Some(args[i].clone());
            }
            "--output" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("ERROR: --output requires a path argument");
                    std::process::exit(1);
                }
                output_path = Some(args[i].clone());
            }
            "--auth" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("ERROR: --auth requires a path argument");
                    std::process::exit(1);
                }
                auth_path = args[i].clone();
            }
            other => {
                eprintln!("ERROR: unknown argument '{}'", other);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    match declaration_path {
        Some(p) => Args { declaration_path: p, output_path, auth_path },
        None => {
            eprintln!("ERROR: --declaration is required");
            eprintln!("Usage: irrs-eval evaluate --declaration <path> [--output <path>] [--auth <path>]");
            std::process::exit(1);
        }
    }
}
