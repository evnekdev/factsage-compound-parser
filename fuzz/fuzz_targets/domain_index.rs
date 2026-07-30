#![no_main]

use factsage_compound_parser::{DatabaseView, DomainIndex, RawDatabase};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_048_576 {
        return;
    }
    if let Ok(raw) = RawDatabase::from_bytes(data) {
        if let Ok(index) = DomainIndex::build(&raw) {
            if let Ok(view) = DatabaseView::new(&raw, &index) {
                for compound in view.compounds() {
                    let _ = compound.name();
                    let _ = compound.formula();
                    for phase in compound.phases() {
                        let _ = phase.name();
                        let _ = phase.compact_label();
                    }
                }
            }
        }
    }
});
