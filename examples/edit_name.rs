use std::error::Error;
use std::path::PathBuf;

use factsage_compound_parser::edit::DatabaseEditor;

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let input = PathBuf::from(
        arguments
            .next()
            .ok_or("usage: edit_name <input.CDB> <output.CDB>")?,
    );
    let output = PathBuf::from(
        arguments
            .next()
            .ok_or("usage: edit_name <input.CDB> <output.CDB>")?,
    );
    let mut editor = DatabaseEditor::from_path(input)?;
    editor.set_compound_name(0, "Example")?;
    editor.write_to_path(output)?;
    Ok(())
}
