use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use factsage_compound_parser::{
    DatabaseEditor, DatabaseView, DiagnosticKind, DomainError, DomainIndex, OrphanReason,
    RangeView, RawChunk, RawDatabase, RawKappaChunk,
};

const CHUNK_SIZE: usize = 256;
const EDIT_SAMPLE_LIMIT: usize = 12;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct SemanticCounts {
    compounds: usize,
    ordinary_phases: usize,
    transition_phases: usize,
    comments: usize,
    linked_cp: usize,
    linked_kappa: usize,
    orphan_kappa: usize,
    ambiguous_kappa: usize,
    phases_with_kappa: usize,
}

#[derive(Debug, Default)]
struct KappaCounts {
    databases: usize,
    chunks: usize,
    compounds: usize,
    phases: usize,
    orphan: usize,
    ambiguous: usize,
    phase_states: BTreeMap<&'static str, usize>,
    phase_id_min: Option<i32>,
    phase_id_max: Option<i32>,
    finite_numeric_fields: usize,
    non_finite_numeric_fields: usize,
    valid_temperature_bounds: usize,
    invalid_temperature_bounds: usize,
    after_cp: usize,
    before_cp: usize,
    before_comment: usize,
    after_comment: usize,
}

#[derive(Debug, Default)]
struct EditorCounts {
    databases_sampled: usize,
    attempted: usize,
    passed: usize,
    skipped: usize,
    failed: usize,
    kappa_bytes_preserved: usize,
}

#[derive(Debug, Default)]
struct CorpusSummary {
    files_discovered: usize,
    files_read: usize,
    parse_successes: usize,
    parse_failures: BTreeMap<&'static str, usize>,
    domain_successes: usize,
    domain_failures: BTreeMap<&'static str, usize>,
    total_bytes: u64,
    total_chunks: usize,
    first_chunk_ids: BTreeMap<u8, usize>,
    magic_matches: usize,
    chunk_ids: BTreeMap<u8, usize>,
    unknown_ids: BTreeMap<u8, usize>,
    diagnostics: BTreeMap<&'static str, usize>,
    cp_ids: BTreeMap<u8, usize>,
    exact_round_trips: usize,
    reader_equivalences: usize,
    deterministic_indexes: usize,
    clone_reuse_rejections: usize,
    from_path_time: Duration,
    from_bytes_time: Duration,
    from_reader_time: Duration,
    index_time: Duration,
    serialize_time: Duration,
    write_time: Duration,
    traversal_time: Duration,
    semantic: SemanticCounts,
    kappa: KappaCounts,
    patterns: BTreeMap<&'static str, usize>,
    editor: EditorCounts,
    file_reports: Vec<String>,
}

#[test]
#[ignore = "requires FACTSAGE_FACTDATA_ROOT and performs read-only local corpus validation"]
fn factsage_installation_corpus_is_lossless_and_structurally_indexable() {
    let Some(configured_root) = std::env::var_os("FACTSAGE_FACTDATA_ROOT") else {
        eprintln!("skipping FactSage corpus validation: FACTSAGE_FACTDATA_ROOT is not set");
        return;
    };
    let root = fs::canonicalize(PathBuf::from(configured_root))
        .expect("configured FactSage corpus root must be readable");
    assert!(root.is_dir(), "configured corpus root must be a directory");

    let mut files = Vec::new();
    discover_cdb_files(&root, &root, &mut files).expect("corpus directory traversal must succeed");
    files.sort();

    let mut summary = CorpusSummary {
        files_discovered: files.len(),
        ..CorpusSummary::default()
    };

    for path in files {
        scan_file(&root, &path, &mut summary);
    }

    write_report(&summary).expect("aggregate report under target must be writable");
    print_summary(&summary);

    assert_eq!(
        summary.editor.failed, 0,
        "editor locality or reparsing failed"
    );
    assert_eq!(
        summary.parse_successes, summary.exact_round_trips,
        "every successfully parsed corpus database must round trip exactly"
    );
    assert_eq!(
        summary.parse_successes, summary.reader_equivalences,
        "from_path, from_bytes, and from_reader must agree"
    );
}

fn discover_cdb_files(root: &Path, directory: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        let canonical = fs::canonicalize(&path)?;
        if !canonical.starts_with(root) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "corpus entry resolves outside the configured root",
            ));
        }
        if file_type.is_dir() {
            discover_cdb_files(root, &canonical, output)?;
        } else if file_type.is_file() && has_cdb_extension(&canonical) {
            output.push(canonical);
        }
    }
    Ok(())
}

fn has_cdb_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("cdb"))
}

fn scan_file(root: &Path, path: &Path, summary: &mut CorpusSummary) {
    if !path.starts_with(root) {
        increment(&mut summary.parse_failures, "outside_configured_root");
        return;
    }
    let relative = path.strip_prefix(root).unwrap_or(path);
    let bytes = match read_only_bytes(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            increment(&mut summary.parse_failures, io_error_category(&error));
            summary.file_reports.push(format!(
                "{} read_error={}",
                relative.display(),
                error.kind()
            ));
            return;
        }
    };
    summary.files_read += 1;
    summary.total_bytes += bytes.len() as u64;
    if let Some(id) = bytes.first().copied() {
        increment(&mut summary.first_chunk_ids, id);
    }
    if bytes.get(2..6) == Some(b"CMPD") {
        summary.magic_matches += 1;
    }

    let parse_started = Instant::now();
    let raw = match RawDatabase::from_path(path) {
        Ok(raw) => raw,
        Err(error) => {
            summary.from_path_time += parse_started.elapsed();
            increment(&mut summary.parse_failures, parse_error_category(&error));
            summary.file_reports.push(format!(
                "{} bytes={} first_id={:?} magic={} parse_error={}",
                relative.display(),
                bytes.len(),
                bytes.first().copied(),
                bytes.get(2..6) == Some(b"CMPD"),
                parse_error_category(&error)
            ));
            return;
        }
    };
    summary.from_path_time += parse_started.elapsed();
    summary.parse_successes += 1;
    summary.total_chunks += raw.chunks().len();
    let file_chunk_ids = chunk_histogram(&raw);
    let file_unknown_ids = unknown_chunk_histogram(&raw);

    let from_bytes_started = Instant::now();
    let from_bytes = match RawDatabase::from_bytes(&bytes) {
        Ok(raw) => raw,
        Err(error) => {
            increment(&mut summary.parse_failures, parse_error_category(&error));
            summary.file_reports.push(format!(
                "{} bytes={} from_bytes_error={}",
                relative.display(),
                bytes.len(),
                parse_error_category(&error)
            ));
            return;
        }
    };
    summary.from_bytes_time += from_bytes_started.elapsed();
    let from_reader_started = Instant::now();
    let from_reader = match RawDatabase::from_reader(Cursor::new(&bytes)) {
        Ok(raw) => raw,
        Err(error) => {
            increment(&mut summary.parse_failures, parse_error_category(&error));
            summary.file_reports.push(format!(
                "{} bytes={} from_reader_error={}",
                relative.display(),
                bytes.len(),
                parse_error_category(&error)
            ));
            return;
        }
    };
    summary.from_reader_time += from_reader_started.elapsed();
    if raw == from_bytes && raw == from_reader {
        summary.reader_equivalences += 1;
    }

    scan_raw(&raw, summary);
    let serialize_started = Instant::now();
    let exact_round_trip = raw.to_bytes().is_ok_and(|serialized| serialized == bytes);
    summary.serialize_time += serialize_started.elapsed();
    if exact_round_trip {
        summary.exact_round_trips += 1;
    }
    let write_started = Instant::now();
    let mut streamed = Vec::new();
    let streaming_matches = raw
        .write_to(&mut streamed)
        .is_ok_and(|()| streamed == bytes);
    summary.write_time += write_started.elapsed();
    if !streaming_matches {
        summary.file_reports.push(format!(
            "{} bytes={} write_to_mismatch",
            relative.display(),
            bytes.len()
        ));
    }

    let index_started = Instant::now();
    let index = match DomainIndex::build(&raw) {
        Ok(index) => index,
        Err(error) => {
            summary.index_time += index_started.elapsed();
            increment(&mut summary.domain_failures, domain_error_category(&error));
            summary.file_reports.push(format!(
                "{} bytes={} first_id={:?} magic={} chunks={} raw_ids={:?} unknown_ids={:?} domain_error={} round_trip={}",
                relative.display(),
                bytes.len(),
                bytes.first().copied(),
                bytes.get(2..6) == Some(b"CMPD"),
                raw.chunks().len(),
                file_chunk_ids,
                file_unknown_ids,
                domain_error_category(&error),
                exact_round_trip
            ));
            return;
        }
    };
    summary.index_time += index_started.elapsed();
    summary.domain_successes += 1;

    let view = DatabaseView::new(&raw, &index).expect("fresh index must create a view");
    let traversal_started = Instant::now();
    let first = collect_semantic(view, summary);
    let second = collect_semantic(view, &mut CorpusSummary::default());
    let rebuilt = DomainIndex::build(&raw).expect("deterministic index rebuild");
    let rebuilt_view = DatabaseView::new(&raw, &rebuilt).expect("rebuilt index must create a view");
    let third = collect_semantic(rebuilt_view, &mut CorpusSummary::default());
    summary.traversal_time += traversal_started.elapsed();
    if first == second && first == third && index.diagnostics() == rebuilt.diagnostics() {
        summary.deterministic_indexes += 1;
    }
    let cloned = raw.clone();
    if DatabaseView::new(&cloned, &index).is_err() {
        summary.clone_reuse_rejections += 1;
    }

    if summary.editor.databases_sampled < EDIT_SAMPLE_LIMIT {
        validate_editor_in_memory(&raw, view, &mut summary.editor);
    }
    let file_diagnostics = diagnostic_histogram(index.diagnostics());
    summary.file_reports.push(format!(
        "{} bytes={} first_id={:?} magic={} chunks={} raw_ids={:?} unknown_ids={:?} compounds={} ordinary={} transition={} cp={} comments={} kappa={} orphan_kappa={} ambiguous_kappa={} diagnostics={:?} domain=ok round_trip={} reader_equivalent={} deterministic_index={}",
        relative.display(),
        bytes.len(),
        bytes.first().copied(),
        bytes.get(2..6) == Some(b"CMPD"),
        raw.chunks().len(),
        file_chunk_ids,
        file_unknown_ids,
        first.compounds,
        first.ordinary_phases,
        first.transition_phases,
        first.linked_cp,
        first.comments,
        first.linked_kappa,
        first.orphan_kappa,
        first.ambiguous_kappa,
        file_diagnostics,
        exact_round_trip,
        raw == from_reader,
        first == second && first == third && index.diagnostics() == rebuilt.diagnostics(),
    ));
}

fn read_only_bytes(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn chunk_histogram(raw: &RawDatabase) -> BTreeMap<u8, usize> {
    let mut histogram = BTreeMap::new();
    for chunk in raw.chunks() {
        increment(&mut histogram, chunk.id());
    }
    histogram
}

fn unknown_chunk_histogram(raw: &RawDatabase) -> BTreeMap<u8, usize> {
    let mut histogram = BTreeMap::new();
    for chunk in raw.chunks() {
        if !(1..=11).contains(&chunk.id()) {
            increment(&mut histogram, chunk.id());
        }
    }
    histogram
}

fn diagnostic_histogram(
    diagnostics: &[factsage_compound_parser::Diagnostic],
) -> BTreeMap<&'static str, usize> {
    let mut histogram = BTreeMap::new();
    for diagnostic in diagnostics {
        increment(&mut histogram, diagnostic_category(&diagnostic.kind));
    }
    histogram
}
fn scan_raw(raw: &RawDatabase, summary: &mut CorpusSummary) {
    let mut contains_kappa = false;
    for (chunk_index, chunk) in raw.chunks().iter().enumerate() {
        let id = chunk.id();
        increment(&mut summary.chunk_ids, id);
        if !(1..=11).contains(&id) {
            increment(&mut summary.unknown_ids, id);
            increment(&mut summary.patterns, "unknown_chunk_id");
        }
        if (2..=6).contains(&id) {
            increment(&mut summary.cp_ids, id);
        }
        match chunk {
            RawChunk::Compound(compound) => {
                if !matches!(compound.unit_energy, 0 | 1) {
                    increment(&mut summary.patterns, "unusual_energy_unit");
                }
                if !matches!(compound.unit_pressure, 0 | 1) {
                    increment(&mut summary.patterns, "unusual_pressure_unit");
                }
                if compound.compound_name.iter().all(|byte| *byte == 0) {
                    increment(&mut summary.patterns, "empty_compound_name");
                }
            }
            RawChunk::HeatCapacity { chunk, .. } => {
                scan_temperature_bounds(
                    chunk.temperature_min,
                    chunk.temperature_max,
                    &mut summary.patterns,
                );
                if chunk_has_non_finite(chunk) {
                    increment(&mut summary.patterns, "non_finite_cp_field");
                }
            }
            RawChunk::Kappa(chunk) => {
                contains_kappa = true;
                scan_kappa(chunk, raw.chunks(), chunk_index, &mut summary.kappa);
                scan_temperature_bounds(
                    chunk.temperature_min,
                    chunk.temperature_max,
                    &mut summary.patterns,
                );
            }
            RawChunk::PhaseOrdinary(chunk) => {
                if !chunk.enthalpy.is_finite()
                    || !chunk.entropy.is_finite()
                    || !chunk.physical.density_raw.is_finite()
                {
                    increment(&mut summary.patterns, "non_finite_phase_field");
                }
            }
            RawChunk::PhaseTransition(chunk) => {
                if !chunk.transition_enthalpy.is_finite()
                    || !chunk.transition_temperature.is_finite()
                    || !chunk.physical.density_raw.is_finite()
                {
                    increment(&mut summary.patterns, "non_finite_phase_field");
                }
            }
            RawChunk::DatabaseHeader(_) | RawChunk::Comment(_) | RawChunk::Unknown { .. } => {}
        }
    }
    if contains_kappa {
        summary.kappa.databases += 1;
    }
}

fn scan_temperature_bounds(t_min: f64, t_max: f64, patterns: &mut BTreeMap<&'static str, usize>) {
    if !t_min.is_finite() || !t_max.is_finite() {
        increment(patterns, "non_finite_temperature_bound");
    } else if t_min > t_max {
        increment(patterns, "reversed_temperature_range");
    } else if t_min == t_max {
        increment(patterns, "zero_width_temperature_range");
    }
}

fn scan_kappa(
    chunk: &RawKappaChunk,
    chunks: &[RawChunk],
    chunk_index: usize,
    kappa: &mut KappaCounts,
) {
    kappa.chunks += 1;
    kappa.phase_id_min = Some(
        kappa
            .phase_id_min
            .map_or(chunk.phase_id_raw, |value| value.min(chunk.phase_id_raw)),
    );
    kappa.phase_id_max = Some(
        kappa
            .phase_id_max
            .map_or(chunk.phase_id_raw, |value| value.max(chunk.phase_id_raw)),
    );
    if chunk.temperature_min.is_finite()
        && chunk.temperature_max.is_finite()
        && chunk.temperature_min <= chunk.temperature_max
    {
        kappa.valid_temperature_bounds += 1;
    } else {
        kappa.invalid_temperature_bounds += 1;
    }
    for value in std::iter::once(chunk.header.timestamp_ole)
        .chain([chunk.temperature_min, chunk.temperature_max])
        .chain(chunk.f1_temperature_coefficients)
        .chain(chunk.f2_pressure_coefficients)
        .chain(chunk.f3_temperature_coefficients)
    {
        increment_finite(
            value.is_finite(),
            &mut kappa.finite_numeric_fields,
            &mut kappa.non_finite_numeric_fields,
        );
    }
    for value in chunk
        .f1_temperature_powers
        .into_iter()
        .chain(chunk.f2_pressure_powers)
        .chain(chunk.f3_temperature_powers)
    {
        increment_finite(
            value.is_finite(),
            &mut kappa.finite_numeric_fields,
            &mut kappa.non_finite_numeric_fields,
        );
    }
    match chunks.get(chunk_index.saturating_sub(1)).map(RawChunk::id) {
        Some(2..=6) => kappa.after_cp += 1,
        Some(10) => kappa.after_comment += 1,
        _ => {}
    }
    match chunks.get(chunk_index.saturating_add(1)).map(RawChunk::id) {
        Some(2..=6) => kappa.before_cp += 1,
        Some(10) => kappa.before_comment += 1,
        _ => {}
    }
}

fn increment_finite(is_finite: bool, finite: &mut usize, non_finite: &mut usize) {
    if is_finite {
        *finite += 1;
    } else {
        *non_finite += 1;
    }
}

fn chunk_has_non_finite(chunk: &factsage_compound_parser::RawHeatCapacityChunk) -> bool {
    !chunk.header.timestamp_ole.is_finite()
        || !chunk.enthalpy.is_finite()
        || !chunk.entropy.is_finite()
        || !chunk.temperature_min.is_finite()
        || !chunk.temperature_max.is_finite()
        || chunk.coefficients.iter().any(|value| !value.is_finite())
        || chunk.powers.iter().any(|value| !value.is_finite())
}

fn collect_semantic(view: DatabaseView<'_>, summary: &mut CorpusSummary) -> SemanticCounts {
    let mut counts = SemanticCounts::default();
    for diagnostic in view.diagnostics() {
        increment(
            &mut summary.diagnostics,
            diagnostic_category(&diagnostic.kind),
        );
    }
    for compound in view.compounds() {
        counts.compounds += 1;
        let mut compound_has_cp = false;
        let mut compound_has_kappa = false;
        let mut transitions = 0;
        counts.comments += compound.comment_fragments().count();
        if compound.comment_fragments().count() > 1 {
            increment(&mut summary.patterns, "multiple_comment_fragments");
        }
        if compound.phase_count() == 0 {
            increment(&mut summary.patterns, "compound_without_phase");
        }
        for orphan in compound.orphan_ranges() {
            if matches!(orphan.range(), Some(RangeView::Kappa(_))) {
                counts.orphan_kappa += 1;
                match orphan.reason() {
                    OrphanReason::MissingPhase => {}
                    OrphanReason::AmbiguousPhase { .. } => counts.ambiguous_kappa += 1,
                }
            }
        }
        for phase in compound.phases() {
            if phase.is_transition() {
                counts.transition_phases += 1;
                transitions += 1;
            } else {
                counts.ordinary_phases += 1;
            }
            let cp_count = phase.heat_capacity_ranges().count();
            let kappa_count = phase.physical_property_ranges().count();
            counts.linked_cp += cp_count;
            counts.linked_kappa += kappa_count;
            compound_has_cp |= cp_count > 0;
            compound_has_kappa |= kappa_count > 0;
            if kappa_count > 0 {
                counts.phases_with_kappa += 1;
                increment(
                    &mut summary.kappa.phase_states,
                    phase_state_name(phase.state()),
                );
            }
        }
        if transitions > 1 {
            increment(&mut summary.patterns, "multiple_transition_phases");
        }
        if compound_has_cp && compound_has_kappa {
            increment(&mut summary.patterns, "compound_with_cp_and_kappa");
        }
        if compound_has_kappa {
            summary.kappa.compounds += 1;
        }
    }
    summary.semantic.compounds += counts.compounds;
    summary.semantic.ordinary_phases += counts.ordinary_phases;
    summary.semantic.transition_phases += counts.transition_phases;
    summary.semantic.comments += counts.comments;
    summary.semantic.linked_cp += counts.linked_cp;
    summary.semantic.linked_kappa += counts.linked_kappa;
    summary.semantic.orphan_kappa += counts.orphan_kappa;
    summary.semantic.ambiguous_kappa += counts.ambiguous_kappa;
    summary.semantic.phases_with_kappa += counts.phases_with_kappa;
    summary.kappa.phases += counts.phases_with_kappa;
    summary.kappa.orphan += counts.orphan_kappa;
    summary.kappa.ambiguous += counts.ambiguous_kappa;
    counts
}

fn validate_editor_in_memory(raw: &RawDatabase, view: DatabaseView<'_>, counts: &mut EditorCounts) {
    let Some(compound) = view.compounds().next() else {
        counts.skipped += 1;
        return;
    };
    counts.databases_sampled += 1;
    let compound_chunk = compound.chunk_index();
    run_edit(
        raw,
        compound_chunk * CHUNK_SIZE + 32..compound_chunk * CHUNK_SIZE + 72,
        |editor| editor.set_compound_name(0, "RC"),
        counts,
    );
    run_edit(
        raw,
        compound_chunk * CHUNK_SIZE + 176..compound_chunk * CHUNK_SIZE + 232,
        |editor| editor.set_real_stoichiometric_coefficients(0, [1.0; 7]),
        counts,
    );

    for (phase_index, phase) in compound.phases().enumerate() {
        let chunk_index = phase.chunk_index();
        run_edit(
            raw,
            chunk_index * CHUNK_SIZE + 136..chunk_index * CHUNK_SIZE + 176,
            move |editor| editor.set_phase_name(0, phase_index, "RC"),
            counts,
        );
        if phase.is_transition() {
            run_edit(
                raw,
                chunk_index * CHUNK_SIZE + 32..chunk_index * CHUNK_SIZE + 40,
                move |editor| editor.set_transition_phase_enthalpy_j_per_mol(0, phase_index, 0.0),
                counts,
            );
            run_edit(
                raw,
                chunk_index * CHUNK_SIZE + 40..chunk_index * CHUNK_SIZE + 48,
                move |editor| editor.set_transition_phase_temperature_k(0, phase_index, 298.15),
                counts,
            );
        } else {
            run_edit(
                raw,
                chunk_index * CHUNK_SIZE + 32..chunk_index * CHUNK_SIZE + 40,
                move |editor| editor.set_ordinary_phase_enthalpy_298_j_per_mol(0, phase_index, 0.0),
                counts,
            );
            run_edit(
                raw,
                chunk_index * CHUNK_SIZE + 40..chunk_index * CHUNK_SIZE + 48,
                move |editor| {
                    editor.set_ordinary_phase_entropy_298_j_per_mol_k(0, phase_index, 0.0)
                },
                counts,
            );
        }
    }
}

fn run_edit<F>(
    raw: &RawDatabase,
    allowed: std::ops::Range<usize>,
    edit: F,
    counts: &mut EditorCounts,
) where
    F: FnOnce(&mut DatabaseEditor) -> Result<(), factsage_compound_parser::EditError>,
{
    counts.attempted += 1;
    let before = match raw.to_bytes() {
        Ok(bytes) => bytes,
        Err(_) => {
            counts.failed += 1;
            return;
        }
    };
    let mut editor = DatabaseEditor::from_raw(raw.clone());
    if editor.view().is_err() {
        counts.failed += 1;
        return;
    }
    if edit(&mut editor).is_err() {
        counts.skipped += 1;
        return;
    }
    let after = match editor.to_bytes() {
        Ok(bytes) => bytes,
        Err(_) => {
            counts.failed += 1;
            return;
        }
    };
    if !changes_only_within(&before, &after, allowed) || RawDatabase::from_bytes(&after).is_err() {
        counts.failed += 1;
        return;
    }
    if editor.view().is_err() {
        counts.failed += 1;
        return;
    }
    for (index, chunk) in raw.chunks().iter().enumerate() {
        if matches!(chunk, RawChunk::Kappa(_)) {
            let start = index * CHUNK_SIZE;
            if before[start..start + CHUNK_SIZE] == after[start..start + CHUNK_SIZE] {
                counts.kappa_bytes_preserved += 1;
            } else {
                counts.failed += 1;
                return;
            }
        }
    }
    counts.passed += 1;
}

fn changes_only_within(before: &[u8], after: &[u8], allowed: std::ops::Range<usize>) -> bool {
    before.len() == after.len()
        && before
            .iter()
            .zip(after)
            .enumerate()
            .all(|(index, (left, right))| left == right || allowed.contains(&index))
}

fn increment<K: Ord>(counts: &mut BTreeMap<K, usize>, key: K) {
    *counts.entry(key).or_default() += 1;
}

fn diagnostic_category(kind: &DiagnosticKind) -> &'static str {
    match kind {
        DiagnosticKind::DuplicatePhaseId { .. } => "duplicate_phase_id",
        DiagnosticKind::OrphanHeatCapacityRange { .. } => "orphan_cp",
        DiagnosticKind::OrphanKappaRange { .. } => "orphan_kappa",
        DiagnosticKind::AmbiguousPhaseLink { .. } => "ambiguous_phase_link",
        DiagnosticKind::InvalidAsciiText { .. } => "invalid_ascii",
        DiagnosticKind::SuspiciousPhaseId { .. } => "suspicious_phase_id",
        DiagnosticKind::NonFiniteTemperatureRange { .. } => "non_finite_temperature_range",
        DiagnosticKind::TemperatureRangeReversed { .. } => "reversed_temperature_range",
        DiagnosticKind::UnknownChunk { .. } => "unknown_chunk",
    }
}

fn parse_error_category(error: &factsage_compound_parser::ParseError) -> &'static str {
    use factsage_compound_parser::ParseError;

    match error {
        ParseError::EmptyFile => "empty_file",
        ParseError::InvalidFileLength { .. } => "invalid_file_length",
        ParseError::InvalidFirstChunkId { .. } => "invalid_first_chunk_id",
        ParseError::InvalidDatabaseMagic { .. } => "invalid_database_magic",
        ParseError::TruncatedRecord { .. } => "truncated_record",
        ParseError::FieldBoundary { .. } => "field_boundary",
        ParseError::Allocation { .. } => "allocation",
        ParseError::RecordCountOverflow => "record_count_overflow",
        ParseError::OpenPath { .. } => "open_path",
        ParseError::ReadPath { .. } => "read_path",
        ParseError::Io(_) => "io",
    }
}

fn domain_error_category(error: &DomainError) -> &'static str {
    match error {
        DomainError::EmptyRawDatabase => "empty_raw_database",
        DomainError::MissingDatabaseHeader { .. } => "missing_database_header",
        DomainError::Ordering { .. } => "ordering",
        DomainError::StaleIndex { .. } => "stale_index",
    }
}

fn io_error_category(error: &io::Error) -> &'static str {
    match error.kind() {
        io::ErrorKind::NotFound => "not_found",
        io::ErrorKind::PermissionDenied => "permission_denied",
        io::ErrorKind::InvalidInput => "invalid_input",
        _ => "io",
    }
}

fn phase_state_name(state: factsage_compound_parser::PhaseState) -> &'static str {
    match state {
        factsage_compound_parser::PhaseState::Solid => "solid",
        factsage_compound_parser::PhaseState::Liquid => "liquid",
        factsage_compound_parser::PhaseState::Gas => "gas",
        factsage_compound_parser::PhaseState::Aqueous => "aqueous",
    }
}

fn write_report(summary: &CorpusSummary) -> io::Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("factsage-corpus-report.txt");
    let mut report = String::new();
    writeln!(report, "files_discovered={}", summary.files_discovered).unwrap();
    writeln!(report, "files_read={}", summary.files_read).unwrap();
    writeln!(report, "total_bytes={}", summary.total_bytes).unwrap();
    writeln!(report, "total_chunks={}", summary.total_chunks).unwrap();
    writeln!(report, "parse_successes={}", summary.parse_successes).unwrap();
    writeln!(report, "domain_successes={}", summary.domain_successes).unwrap();
    writeln!(report, "exact_round_trips={}", summary.exact_round_trips).unwrap();
    writeln!(
        report,
        "reader_equivalences={}",
        summary.reader_equivalences
    )
    .unwrap();
    writeln!(
        report,
        "deterministic_indexes={}",
        summary.deterministic_indexes
    )
    .unwrap();
    writeln!(
        report,
        "clone_reuse_rejections={}",
        summary.clone_reuse_rejections
    )
    .unwrap();
    writeln!(report, "chunk_ids={:?}", summary.chunk_ids).unwrap();
    writeln!(report, "unknown_ids={:?}", summary.unknown_ids).unwrap();
    writeln!(report, "cp_ids={:?}", summary.cp_ids).unwrap();
    writeln!(report, "parse_failures={:?}", summary.parse_failures).unwrap();
    writeln!(report, "domain_failures={:?}", summary.domain_failures).unwrap();
    writeln!(report, "diagnostics={:?}", summary.diagnostics).unwrap();
    writeln!(report, "patterns={:?}", summary.patterns).unwrap();
    writeln!(report, "kappa_databases={}", summary.kappa.databases).unwrap();
    writeln!(report, "kappa_chunks={}", summary.kappa.chunks).unwrap();
    writeln!(report, "kappa_compounds={}", summary.kappa.compounds).unwrap();
    writeln!(report, "kappa_phases={}", summary.kappa.phases).unwrap();
    writeln!(report, "kappa_orphan={}", summary.kappa.orphan).unwrap();
    writeln!(report, "kappa_ambiguous={}", summary.kappa.ambiguous).unwrap();
    writeln!(
        report,
        "kappa_phase_states={:?}",
        summary.kappa.phase_states
    )
    .unwrap();
    writeln!(
        report,
        "kappa_phase_id_min={:?}",
        summary.kappa.phase_id_min
    )
    .unwrap();
    writeln!(
        report,
        "kappa_phase_id_max={:?}",
        summary.kappa.phase_id_max
    )
    .unwrap();
    writeln!(
        report,
        "kappa_finite_numeric_fields={}",
        summary.kappa.finite_numeric_fields
    )
    .unwrap();
    writeln!(
        report,
        "kappa_non_finite_numeric_fields={}",
        summary.kappa.non_finite_numeric_fields
    )
    .unwrap();
    writeln!(
        report,
        "kappa_valid_temperature_bounds={}",
        summary.kappa.valid_temperature_bounds
    )
    .unwrap();
    writeln!(
        report,
        "kappa_invalid_temperature_bounds={}",
        summary.kappa.invalid_temperature_bounds
    )
    .unwrap();
    writeln!(report, "kappa_order_after_cp={}", summary.kappa.after_cp).unwrap();
    writeln!(report, "kappa_order_before_cp={}", summary.kappa.before_cp).unwrap();
    writeln!(
        report,
        "kappa_order_before_comment={}",
        summary.kappa.before_comment
    )
    .unwrap();
    writeln!(
        report,
        "kappa_order_after_comment={}",
        summary.kappa.after_comment
    )
    .unwrap();
    writeln!(report, "editor={:?}", summary.editor).unwrap();
    writeln!(
        report,
        "from_path_us={}",
        summary.from_path_time.as_micros()
    )
    .unwrap();
    writeln!(
        report,
        "from_bytes_us={}",
        summary.from_bytes_time.as_micros()
    )
    .unwrap();
    writeln!(
        report,
        "from_reader_us={}",
        summary.from_reader_time.as_micros()
    )
    .unwrap();
    writeln!(report, "index_us={}", summary.index_time.as_micros()).unwrap();
    writeln!(
        report,
        "traversal_us={}",
        summary.traversal_time.as_micros()
    )
    .unwrap();
    writeln!(report, "to_bytes_us={}", summary.serialize_time.as_micros()).unwrap();
    writeln!(report, "write_to_us={}", summary.write_time.as_micros()).unwrap();
    writeln!(report, "files:").unwrap();
    for line in &summary.file_reports {
        writeln!(report, "{line}").unwrap();
    }
    fs::write(path, report)
}

fn print_summary(summary: &CorpusSummary) {
    println!(
        "files={} bytes={} chunks={} parse_ok={} domain_ok={} round_trip_ok={} reader_equivalent={} kappa_files={} kappa_chunks={} kappa_orphan={} kappa_ambiguous={} parse_failures={:?} domain_failures={:?} unknown_ids={:?} patterns={:?} editor_attempted={} editor_passed={} editor_skipped={} editor_failed={}",
        summary.files_discovered,
        summary.total_bytes,
        summary.total_chunks,
        summary.parse_successes,
        summary.domain_successes,
        summary.exact_round_trips,
        summary.reader_equivalences,
        summary.kappa.databases,
        summary.kappa.chunks,
        summary.kappa.orphan,
        summary.kappa.ambiguous,
        summary.parse_failures,
        summary.domain_failures,
        summary.unknown_ids,
        summary.patterns,
        summary.editor.attempted,
        summary.editor.passed,
        summary.editor.skipped,
        summary.editor.failed,
    );
}
