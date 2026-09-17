use companion_policy::{TomlToolRegistry, ToolCapabilityResolver, ToolCatalogSnapshot};

#[test]
fn embedded_tool_registry_is_valid() {
    let _registry = TomlToolRegistry::embedded();
}

#[test]
fn embedded_tool_registry_resolves_a_known_seed_entry() {
    let registry = TomlToolRegistry::embedded();
    assert!(registry.resolve("sch_add_symbol").is_some());
}

#[test]
fn embedded_tool_registry_classifies_manufacturing_export_as_high_risk() {
    let registry = TomlToolRegistry::embedded();
    let (capability, risk) = registry
        .resolve("export_gerber")
        .expect("export_gerber is a known tool");
    assert_eq!(capability, companion_core::Capability::MANUFACTURING_EXPORT);
    assert_eq!(risk, companion_core::RiskLevel::High);
}

#[test]
fn embedded_tool_registry_denies_unknown_tool() {
    let registry = TomlToolRegistry::embedded();
    assert_eq!(registry.resolve("shell.exec"), None);
}

#[test]
fn coverage_report_separates_classified_unclassified_and_stale_tools() {
    let registry = TomlToolRegistry::from_toml_str(
        r#"
        [[tool]]
        name = "known_tool"
        capability = "schematic.read"
        risk = "low"

        [[tool]]
        name = "stale_tool"
        capability = "schematic.read"
        risk = "low"
        "#,
    )
    .unwrap();
    let snapshot = ToolCatalogSnapshot::from_tool_names(
        "oaslananka/kicad-mcp-pro",
        "main",
        "deadbeef",
        ["known_tool", "new_tool"],
    );

    let report = registry.coverage_against(&snapshot);
    assert_eq!(report.classified, vec!["known_tool"]);
    assert_eq!(report.unclassified, vec!["new_tool"]);
    assert_eq!(report.stale, vec!["stale_tool"]);
    assert_eq!(report.catalog_total, 2);
    assert_eq!(report.registry_total, 2);
}

#[test]
fn embedded_registry_has_no_entries_missing_from_upstream_snapshot() {
    let registry = TomlToolRegistry::embedded();
    let snapshot = ToolCatalogSnapshot::embedded();
    let report = registry.coverage_against(&snapshot);
    assert!(
        report.stale.is_empty(),
        "stale registry entries: {:?}",
        report.stale
    );
}

#[test]
fn unclassified_upstream_tool_remains_fail_closed() {
    let registry = TomlToolRegistry::embedded();
    let snapshot = ToolCatalogSnapshot::embedded();
    assert!(snapshot.contains("kicad_get_version"));
    assert_eq!(registry.resolve("kicad_get_version"), None);
}
