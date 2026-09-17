use std::error::Error;
use std::path::PathBuf;

use companion_policy::{TomlToolRegistry, ToolCatalogSnapshot};

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if !(2..=3).contains(&args.len()) {
        return Err(
            "usage: reconcile-tool-registry <tools-reference.generated.md> <source-sha> [output.toml]"
                .into(),
        );
    }

    let source_path = PathBuf::from(&args[0]);
    let source_sha = &args[1];
    let markdown = std::fs::read_to_string(&source_path)?;
    let snapshot = ToolCatalogSnapshot::from_tools_reference_markdown(
        "oaslananka/kicad-mcp-pro",
        "main",
        source_sha,
        &markdown,
    )?;
    let registry = TomlToolRegistry::embedded();
    let report = registry.coverage_against(&snapshot);

    let snapshot_toml = snapshot.to_toml_pretty()?;
    if let Some(output) = args.get(2) {
        std::fs::write(output, snapshot_toml)?;
    } else {
        print!("{snapshot_toml}");
    }

    println!(
        "catalog={} registry={} classified={} unclassified={} stale={}",
        report.catalog_total,
        report.registry_total,
        report.classified.len(),
        report.unclassified.len(),
        report.stale.len()
    );
    if !report.stale.is_empty() {
        eprintln!("stale registry entries: {}", report.stale.join(", "));
    }
    Ok(())
}
