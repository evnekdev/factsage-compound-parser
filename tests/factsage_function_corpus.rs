use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use factsage_compound_parser::{
    CompoundDatabaseProfileEvidence, Database, PhaseThermodynamicView, PhaseThermodynamicViewError,
};

/// Validates the local FDB corpus without copying, printing, or committing any
/// proprietary record content. Set `FACTSAGE_FUNCTIONDATA_ROOT` to the
/// FactSage installation directory containing the files.
#[test]
#[ignore = "requires FACTSAGE_FUNCTIONDATA_ROOT and reads local FDB files read-only"]
fn installed_fdb_corpus_has_profile_compatible_ordinary_views() {
    let Some(configured_root) = std::env::var_os("FACTSAGE_FUNCTIONDATA_ROOT") else {
        return;
    };
    let root = fs::canonicalize(PathBuf::from(configured_root))
        .expect("configured FDB corpus root must be readable");
    let mut files = Vec::new();
    discover_fdb_files(&root, &root, &mut files).expect("FDB corpus traversal must succeed");
    assert!(!files.is_empty(), "configured root contains no FDB files");

    let mut ordinary_views = 0_usize;
    let mut transition_views = 0_usize;
    let mut phases_without_cp = 0_usize;
    let mut degenerate_ranges = 0_usize;
    let mut invalid_ordinary_views = 0_usize;

    for path in files {
        let database = Database::from_path(&path).expect("installed FDB must parse and index");
        let view = database.view().expect("fresh database must create a view");
        assert_eq!(
            view.database_profile_evidence(),
            CompoundDatabaseProfileEvidence::FunctionCompatible,
            "installed FDB must satisfy the observed header guardrail"
        );
        for compound in view.compounds() {
            for phase_index in 0..compound.phase_count() {
                match compound.fdb_phase_thermodynamic_view(phase_index) {
                    Ok(PhaseThermodynamicView::Ordinary(_)) => ordinary_views += 1,
                    Ok(PhaseThermodynamicView::Transition(_)) => transition_views += 1,
                    Err(PhaseThermodynamicViewError::MissingHeatCapacityRanges { .. }) => {
                        phases_without_cp += 1;
                    }
                    Err(PhaseThermodynamicViewError::InvalidTemperatureRange { .. }) => {
                        degenerate_ranges += 1;
                    }
                    Err(_) => invalid_ordinary_views += 1,
                }
            }
        }
    }

    assert!(
        ordinary_views > 0,
        "installed FDB corpus has no CP-backed phase"
    );
    assert_eq!(
        invalid_ordinary_views, 0,
        "FDB CP-backed ordinary phases must not silently recover structural errors"
    );
    let _ = (transition_views, phases_without_cp, degenerate_ranges);
}

fn discover_fdb_files(root: &Path, directory: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let path = fs::canonicalize(entry.path())?;
        if !path.starts_with(root) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "corpus entry resolves outside the configured root",
            ));
        }
        if file_type.is_dir() {
            discover_fdb_files(root, &path, output)?;
        } else if file_type.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("fdb"))
        {
            output.push(path);
        }
    }
    Ok(())
}
