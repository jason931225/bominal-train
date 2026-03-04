use std::{
    env,
    path::{Path, PathBuf},
};

use bominal_shared::station_catalog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Check,
    Generate,
    Validate,
}

#[derive(Debug)]
struct CliArgs {
    command: Command,
    snapshot_path: PathBuf,
    metadata_path: PathBuf,
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<(), String> {
    let args = parse_args(env::args().collect())?;
    match args.command {
        Command::Check => run_check(&args).await,
        Command::Generate => run_generate(&args).await,
        Command::Validate => run_validate(&args),
    }
}

async fn run_check(args: &CliArgs) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|err| format!("failed to build http client: {err}"))?;

    let source = station_catalog::fetch_station_source(&client)
        .await
        .map_err(|err| format!("failed to fetch station source: {err}"))?;
    let source_sha256 = station_catalog::compute_sha256_hex(source.as_bytes());

    let changed = match station_catalog::load_metadata(&args.metadata_path) {
        Ok(metadata) => metadata.source_sha256 != source_sha256,
        Err(_) => true,
    };

    println!("changed={changed}");
    println!("source_sha256={source_sha256}");
    Ok(())
}

async fn run_generate(args: &CliArgs) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|err| format!("failed to build http client: {err}"))?;

    let source = station_catalog::fetch_station_source(&client)
        .await
        .map_err(|err| format!("failed to fetch station source: {err}"))?;
    let generated = station_catalog::generate_from_source(&source)
        .map_err(|err| format!("failed to parse source: {err}"))?;

    station_catalog::write_generated_catalog(&args.snapshot_path, &args.metadata_path, &generated)
        .map_err(|err| format!("failed to write generated catalog: {err}"))?;

    println!("wrote_snapshot={}", args.snapshot_path.display());
    println!("wrote_metadata={}", args.metadata_path.display());
    println!("station_count={}", generated.snapshot.stations.len());
    println!("source_sha256={}", generated.metadata.source_sha256);
    Ok(())
}

fn run_validate(args: &CliArgs) -> Result<(), String> {
    let (snapshot, snapshot_sha256) = station_catalog::load_snapshot_with_hash(&args.snapshot_path)
        .map_err(|err| format!("failed to load snapshot: {err}"))?;
    let metadata = station_catalog::load_metadata(&args.metadata_path)
        .map_err(|err| format!("failed to load metadata: {err}"))?;

    if metadata.schema_version != snapshot.schema_version {
        return Err(format!(
            "schema version mismatch (snapshot={}, metadata={})",
            snapshot.schema_version, metadata.schema_version
        ));
    }
    if metadata.generated_at != snapshot.generated_at {
        return Err("generated_at mismatch between snapshot and metadata".to_string());
    }
    if metadata.source_url.trim().is_empty() {
        return Err("metadata source_url is required".to_string());
    }

    println!("snapshot_path={}", args.snapshot_path.display());
    println!("metadata_path={}", args.metadata_path.display());
    println!("snapshot_sha256={snapshot_sha256}");
    println!("source_sha256={}", metadata.source_sha256);
    println!("station_count={}", snapshot.stations.len());
    Ok(())
}

fn parse_args(raw_args: Vec<String>) -> Result<CliArgs, String> {
    if raw_args.len() < 2 {
        return Err(usage("missing command"));
    }

    let command = match raw_args[1].as_str() {
        "check" => Command::Check,
        "generate" => Command::Generate,
        "validate" => Command::Validate,
        _ => return Err(usage("command must be one of: check, generate, validate")),
    };

    let mut snapshot_path = env::var("STATION_CATALOG_JSON_PATH")
        .unwrap_or_else(|_| "data/train/station_catalog.v1.json".to_string());
    let mut metadata_path = env::var("STATION_CATALOG_METADATA_PATH")
        .unwrap_or_else(|_| "data/train/station_catalog.meta.json".to_string());

    let mut idx = 2usize;
    while idx < raw_args.len() {
        match raw_args[idx].as_str() {
            "--snapshot" => {
                let value = raw_args
                    .get(idx + 1)
                    .ok_or_else(|| usage("--snapshot requires a value"))?;
                snapshot_path = value.clone();
                idx += 2;
            }
            "--meta" => {
                let value = raw_args
                    .get(idx + 1)
                    .ok_or_else(|| usage("--meta requires a value"))?;
                metadata_path = value.clone();
                idx += 2;
            }
            "--help" | "-h" => {
                return Err(usage(""));
            }
            unknown => {
                return Err(usage(&format!("unknown argument: {unknown}")));
            }
        }
    }

    Ok(CliArgs {
        command,
        snapshot_path: resolve_path(&snapshot_path),
        metadata_path: resolve_path(&metadata_path),
    })
}

fn resolve_path(configured_path: &str) -> PathBuf {
    let direct = Path::new(configured_path);
    if direct.exists() {
        return direct.to_path_buf();
    }

    if direct.is_absolute() {
        return direct.to_path_buf();
    }

    let cwd = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let candidates = [
        cwd.join(direct),
        cwd.join("..").join(direct),
        cwd.join("runtime").join(direct),
    ];
    for candidate in candidates {
        if candidate.exists() {
            return candidate;
        }
    }

    direct.to_path_buf()
}

fn usage(message: &str) -> String {
    let mut out = String::new();
    if !message.is_empty() {
        out.push_str(message);
        out.push('\n');
        out.push('\n');
    }
    out.push_str(
        "usage: station_catalog_sync <check|generate|validate> [--snapshot <path>] [--meta <path>]\n",
    );
    out
}
