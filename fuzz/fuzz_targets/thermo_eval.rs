#![no_main]

use factsage_compound_parser::Database;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_048_576 {
        return;
    }
    if let Ok(database) = Database::from_bytes(data) {
        if let Ok(view) = database.view() {
            for compound in view.compounds() {
                for phase in compound.phases() {
                    let _ = phase.density();
                    for range in phase.heat_capacity_ranges() {
                        let t_min = range.temperature_min();
                        let t_max = range.temperature_max();
                        if t_min.is_finite() && t_max.is_finite() && t_min <= t_max && t_max > 0.0 {
                            let temperature = if t_min > 0.0 {
                                t_min + (t_max - t_min) / 2.0
                            } else {
                                t_max / 2.0
                            };
                            if temperature.is_finite() && temperature > 0.0 {
                                let _ = range.heat_capacity_raw(temperature);
                            }
                        }
                    }
                }
            }
        }
    }
});
