use std::error::Error;
use std::path::PathBuf;

use factsage_compound_parser::domain::Database;

fn main() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("usage: heat_capacity <file.CDB>")?,
    );
    let database = Database::from_path(path)?;
    let view = database.view()?;
    let compound = view.compounds().next().ok_or("database has no compounds")?;
    println!("Cp(1000 K) = {}", compound.heat_capacity_at(0, 1000.0)?);
    Ok(())
}
