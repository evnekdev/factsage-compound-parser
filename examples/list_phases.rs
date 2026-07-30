use std::error::Error;
use std::path::PathBuf;

use factsage_compound_parser::domain::Database;

fn main() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("usage: list_phases <file.CDB>")?,
    );
    let database = Database::from_path(path)?;
    for compound in &database.compounds {
        println!("{}", compound.name()?);
        for phase in &compound.phases {
            println!("  {}: {}", phase.chemapp_label(), phase.name()?);
        }
    }
    Ok(())
}
