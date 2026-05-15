use rcount_core::{LineageKind, RcountPackage, ReportingUnitLineage};
use rhist_core::{LineageConfidence, LineageEvent, LineageEventKind};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RcountRhistError {
    #[error("missing effective date for RHIST cycle: {cycle_id}")]
    MissingEffectiveDate { cycle_id: String },
}

pub fn map_lineage_event(
    event: &ReportingUnitLineage,
    effective_date: impl Into<String>,
    confidence: LineageConfidence,
    source_refs: Vec<String>,
) -> LineageEvent {
    LineageEvent {
        event_id: event.lineage_id.clone(),
        event_kind: map_lineage_kind(event.kind.clone()),
        from_cycle_id: event.prior_cycle.clone(),
        to_cycle_id: event.current_cycle.clone(),
        from_unit_ids: event.prior_reporting_unit_ids.clone(),
        to_unit_ids: event.current_reporting_unit_ids.clone(),
        effective_date: effective_date.into(),
        authority: event.authority.clone(),
        confidence,
        source_refs,
        explanation: event.explanation.clone(),
    }
}

pub fn map_package_lineage(
    package: &RcountPackage,
    effective_dates_by_cycle: &BTreeMap<String, String>,
) -> Result<Vec<LineageEvent>, RcountRhistError> {
    package
        .lineage
        .iter()
        .map(|event| {
            let effective_date = effective_dates_by_cycle
                .get(&event.current_cycle)
                .ok_or_else(|| RcountRhistError::MissingEffectiveDate {
                    cycle_id: event.current_cycle.clone(),
                })?;
            Ok(map_lineage_event(
                event,
                effective_date.clone(),
                LineageConfidence::Official,
                Vec::new(),
            ))
        })
        .collect()
}

fn map_lineage_kind(kind: LineageKind) -> LineageEventKind {
    match kind {
        LineageKind::Unchanged => LineageEventKind::Unchanged,
        LineageKind::Split => LineageEventKind::Split,
        LineageKind::Merge => LineageEventKind::Merge,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcount_core::synthetic_precinct_split_lineage_package;
    use rhist_core::{
        package_content_hash, verify_package, ClaimBoundary, ContextIndexEntry, CycleKind,
        CycleRecord, RhistManifest, RhistPackage, RHIST_VERSION,
    };

    #[test]
    fn maps_rcount_lineage_events_to_rhist_events() {
        let package = synthetic_precinct_split_lineage_package();
        let events = map_package_lineage(&package, &effective_dates()).unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_id, "lineage:P-004-split");
        assert_eq!(events[0].event_kind, LineageEventKind::Split);
        assert_eq!(events[0].from_unit_ids, vec!["syn:precinct:P-004"]);
        assert_eq!(
            events[0].to_unit_ids,
            vec!["syn:precinct:P-004A", "syn:precinct:P-004B"]
        );
        assert_eq!(events[1].event_kind, LineageEventKind::Merge);
    }

    #[test]
    fn mapped_rcount_lineage_verifies_as_rhist_package() {
        let rcount = synthetic_precinct_split_lineage_package();
        let events = map_package_lineage(&rcount, &effective_dates()).unwrap();
        let package = rhist_package_for_mapped_events(events);

        let reports = verify_package(&package).expect("mapped RCOUNT lineage should verify");
        assert!(reports
            .iter()
            .any(|report| report.check_id == "lineage_unit_refs"));
        assert!(reports
            .iter()
            .any(|report| report.check_id == "lineage_cardinality"));
    }

    #[test]
    fn mapping_requires_effective_date_for_current_cycle() {
        let package = synthetic_precinct_split_lineage_package();
        let err = map_package_lineage(&package, &BTreeMap::new()).unwrap_err();
        assert_eq!(
            err,
            RcountRhistError::MissingEffectiveDate {
                cycle_id: "SYN-2028-general".to_string()
            }
        );
    }

    fn effective_dates() -> BTreeMap<String, String> {
        BTreeMap::from([("SYN-2028-general".to_string(), "2028-11-07".to_string())])
    }

    fn rhist_package_for_mapped_events(events: Vec<LineageEvent>) -> RhistPackage {
        let mut package = RhistPackage {
            manifest: RhistManifest {
                rhist_version: RHIST_VERSION.to_string(),
                package_id: "syn-rhist-from-rcount-lineage".to_string(),
                jurisdiction: "SYN".to_string(),
                cycle_ids: vec![
                    "SYN-2024-general".to_string(),
                    "SYN-2028-general".to_string(),
                ],
                producer: "rcount-rhist-test".to_string(),
                created_at: "2026-05-13T00:00:00Z".to_string(),
                package_content_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_string(),
            },
            source_index: Vec::new(),
            context_index: vec![
                ContextIndexEntry {
                    context_id: "syn-2024-precinct-context".to_string(),
                    context_hash:
                        "sha256:5555555555555555555555555555555555555555555555555555555555555555"
                            .to_string(),
                    rctx_version: "0.1".to_string(),
                    unit_kind: "precinct".to_string(),
                    cycle_id: "SYN-2024-general".to_string(),
                    unit_ids: vec![
                        "syn:precinct:P-004".to_string(),
                        "syn:precinct:P-007".to_string(),
                        "syn:precinct:P-008".to_string(),
                    ],
                    source_refs: Vec::new(),
                },
                ContextIndexEntry {
                    context_id: "syn-2028-precinct-context".to_string(),
                    context_hash:
                        "sha256:6666666666666666666666666666666666666666666666666666666666666666"
                            .to_string(),
                    rctx_version: "0.1".to_string(),
                    unit_kind: "precinct".to_string(),
                    cycle_id: "SYN-2028-general".to_string(),
                    unit_ids: vec![
                        "syn:precinct:P-004A".to_string(),
                        "syn:precinct:P-004B".to_string(),
                        "syn:precinct:P-078".to_string(),
                    ],
                    source_refs: Vec::new(),
                },
            ],
            cycles: vec![
                CycleRecord {
                    cycle_id: "SYN-2024-general".to_string(),
                    jurisdiction: "SYN".to_string(),
                    cycle_kind: CycleKind::GeneralElection,
                    effective_date: "2024-11-05".to_string(),
                    context_id: "syn-2024-precinct-context".to_string(),
                    context_hash:
                        "sha256:5555555555555555555555555555555555555555555555555555555555555555"
                            .to_string(),
                    source_refs: Vec::new(),
                },
                CycleRecord {
                    cycle_id: "SYN-2028-general".to_string(),
                    jurisdiction: "SYN".to_string(),
                    cycle_kind: CycleKind::GeneralElection,
                    effective_date: "2028-11-07".to_string(),
                    context_id: "syn-2028-precinct-context".to_string(),
                    context_hash:
                        "sha256:6666666666666666666666666666666666666666666666666666666666666666"
                            .to_string(),
                    source_refs: Vec::new(),
                },
            ],
            lineage_events: events,
            crosswalks: Vec::new(),
            claim_boundary: ClaimBoundary {
                package_id: "syn-rhist-from-rcount-lineage".to_string(),
                proves: vec!["mapped RCOUNT lineage is RHIST-compatible".to_string()],
                does_not_prove: vec![
                    "source completeness".to_string(),
                    "vote totals or district assignments".to_string(),
                ],
                caveats: vec![
                    "test package uses synthetic contexts and no crosswalk weights".to_string(),
                ],
            },
        };
        package.manifest.package_content_hash = package_content_hash(&package).unwrap();
        package
    }
}
