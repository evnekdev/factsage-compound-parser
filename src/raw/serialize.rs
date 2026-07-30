use std::fmt;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::{BODY_SIZE, CHUNK_SIZE, RawDatabase};

use super::{
    RawChunk, RawCommentChunk, RawCommonHeader, RawCompoundChunk, RawDatabaseHeaderChunk,
    RawHeatCapacityChunk, RawKappaChunk, RawOrdinaryPhaseChunk, RawPhasePhysicalTail,
    RawTransitionPhaseChunk,
};

/// Errors raised while serializing a raw CDB stream.
#[derive(Debug)]
pub enum SerializeError {
    /// The requested output length cannot be represented by this process.
    LengthOverflow {
        /// Number of fixed-size chunks in the database.
        chunk_count: usize,
    },
    /// Allocating the in-memory byte output failed.
    Allocation {
        /// Number of bytes requested for the output.
        byte_length: usize,
    },
    /// Creating a caller-selected output path failed before any CDB bytes were written.
    CreatePath {
        /// Path that serialization attempted to create or replace.
        path: PathBuf,
        /// Operating-system error returned while creating the path.
        source: io::Error,
    },
    /// Writing a caller-selected output path failed after it was opened.
    ///
    /// The destination can contain a partial CDB stream when this error occurs.
    WritePath {
        /// Path whose write operation failed.
        path: PathBuf,
        /// Operating-system error returned while writing the path.
        source: io::Error,
    },
    /// The destination reader or writer returned an I/O error.
    Io(io::Error),
}

impl fmt::Display for SerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow { chunk_count } => write!(
                formatter,
                "serialized length overflows usize for {chunk_count} CDB chunks"
            ),
            Self::Allocation { byte_length } => {
                write!(formatter, "could not allocate {byte_length} output bytes")
            }
            Self::CreatePath { path, source } => write!(
                formatter,
                "could not create CDB output path {}: {}",
                path.display(),
                source
            ),
            Self::WritePath { path, source } => write!(
                formatter,
                "CDB serialization I/O error at path {}: {}",
                path.display(),
                source
            ),
            Self::Io(error) => write!(formatter, "CDB serialization I/O error: {error}"),
        }
    }
}

impl std::error::Error for SerializeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreatePath { source, .. } | Self::WritePath { source, .. } => Some(source),
            Self::Io(error) => Some(error),
            Self::LengthOverflow { .. } | Self::Allocation { .. } => None,
        }
    }
}

impl From<io::Error> for SerializeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl RawDatabase {
    /// Serializes this flat raw database without altering chunk order or bodies.
    ///
    /// For an unmodified database parsed by [`RawDatabase::from_bytes`], the
    /// returned bytes are identical to the input, including floating-point bit
    /// patterns, fixed-width text, padding, reserved fields, and unknown chunks.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerializeError> {
        let byte_length =
            self.chunks
                .len()
                .checked_mul(CHUNK_SIZE)
                .ok_or(SerializeError::LengthOverflow {
                    chunk_count: self.chunks.len(),
                })?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(byte_length)
            .map_err(|_| SerializeError::Allocation { byte_length })?;
        self.write_to(&mut bytes)?;
        Ok(bytes)
    }

    /// Writes this flat raw database to a byte sink without allocating an output buffer.
    ///
    /// Chunks are emitted in physical stream order. A sink failure is returned as
    /// [`SerializeError::Io`] and may leave the sink with a partial CDB stream.
    pub fn write_to<W: Write>(&self, mut writer: W) -> Result<(), SerializeError> {
        for chunk in &self.chunks {
            chunk.write_to(&mut writer)?;
        }
        Ok(())
    }

    /// Creates or replaces a caller-selected file with this serialized raw database.
    ///
    /// Callers choose the destination explicitly; this method never writes an
    /// input path implicitly. If an operating-system write error occurs after
    /// creation, the destination can contain a partial CDB stream and the error
    /// is returned as [`SerializeError::WritePath`] with its underlying source.
    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), SerializeError> {
        let path = path.as_ref();
        let file = File::create(path).map_err(|source| SerializeError::CreatePath {
            path: path.to_path_buf(),
            source,
        })?;
        match self.write_to(file) {
            Err(SerializeError::Io(source)) => Err(SerializeError::WritePath {
                path: path.to_path_buf(),
                source,
            }),
            result => result,
        }
    }
}

impl RawChunk {
    /// Writes this exact raw chunk, including its original ID byte.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), SerializeError> {
        writer.write_all(&[self.id()])?;
        match self {
            Self::Compound(chunk) => write_compound(writer, chunk),
            Self::HeatCapacity { chunk, .. } => write_heat_capacity(writer, chunk),
            Self::PhaseOrdinary(chunk) => write_ordinary_phase(writer, chunk),
            Self::PhaseTransition(chunk) => write_transition_phase(writer, chunk),
            Self::DatabaseHeader(chunk) => write_database_header(writer, chunk),
            Self::Comment(chunk) => write_comment(writer, chunk),
            Self::Kappa(chunk) => write_kappa(writer, chunk),
            Self::Unknown { body, .. } => writer.write_all(body).map_err(SerializeError::from),
        }
    }
}

fn write_common_header<W: Write>(
    writer: &mut W,
    header: &RawCommonHeader,
) -> Result<(), SerializeError> {
    write_u8_array(writer, &header.element_ids)?;
    write_u8(writer, header.coefficient_padding)?;
    write_u8_array(writer, &header.element_coefficients)?;
    write_i8(writer, header.charge_raw)?;
    write_u8(writer, header.entry_number)?;
    write_u16_array(writer, &header.reference)?;
    write_f64(writer, header.timestamp_ole)?;
    write_u8_array(writer, &header.unknown)
}

fn write_database_header<W: Write>(
    writer: &mut W,
    chunk: &RawDatabaseHeaderChunk,
) -> Result<(), SerializeError> {
    write_u8(writer, chunk.padding_1)?;
    write_u8_array(writer, &chunk.magic)?;
    write_u8_array(writer, &chunk.padding_2)?;
    write_f64(writer, chunk.date_ole)?;
    write_u8(writer, chunk.read_flag)?;
    write_u8_array(writer, &chunk.unknown_1)?;
    write_u8_array(writer, &chunk.comment)?;
    write_u8_array(writer, &chunk.padding_3)?;
    write_u8_array(writer, &chunk.unknown_2)
}

fn write_compound<W: Write>(
    writer: &mut W,
    chunk: &RawCompoundChunk,
) -> Result<(), SerializeError> {
    write_common_header(writer, &chunk.header)?;
    write_u8_array(writer, &chunk.compound_name)?;
    write_u8_array(writer, &chunk.reserved_string_1)?;
    write_u8_array(writer, &chunk.formula_name)?;
    write_u8_array(writer, &chunk.unknown)?;
    write_u32(writer, chunk.unit_energy)?;
    write_u32(writer, chunk.unit_pressure)?;
    write_u8_array(writer, &chunk.reserved_string_2)?;
    write_f64_array(writer, &chunk.real_stoichiometric_coefficients)?;
    write_u8_array(writer, &chunk.padding_final)
}

fn write_ordinary_phase<W: Write>(
    writer: &mut W,
    chunk: &RawOrdinaryPhaseChunk,
) -> Result<(), SerializeError> {
    write_common_header(writer, &chunk.header)?;
    write_f64(writer, chunk.enthalpy)?;
    write_f64(writer, chunk.entropy)?;
    write_i32(writer, chunk.phase_id_raw_neg)?;
    write_i32(writer, chunk.phase_id_raw)?;
    write_phase_physical_tail(writer, &chunk.physical)
}

fn write_transition_phase<W: Write>(
    writer: &mut W,
    chunk: &RawTransitionPhaseChunk,
) -> Result<(), SerializeError> {
    write_common_header(writer, &chunk.header)?;
    write_f64(writer, chunk.transition_enthalpy)?;
    write_f64(writer, chunk.transition_temperature)?;
    write_i32(writer, chunk.parent_phase_id_raw)?;
    write_i32(writer, chunk.phase_id_raw)?;
    write_phase_physical_tail(writer, &chunk.physical)
}

fn write_phase_physical_tail<W: Write>(
    writer: &mut W,
    tail: &RawPhasePhysicalTail,
) -> Result<(), SerializeError> {
    write_f64(writer, tail.density_raw)?;
    write_f32_array(writer, &tail.thermal_expansion_coefficients)?;
    write_f32_array(writer, &tail.compressibility_coefficients)?;
    write_f32_array(writer, &tail.bulk_modulus_derivative_coefficients)?;
    write_f32(writer, tail.magnetic_temperature)?;
    write_f32(writer, tail.magnetic_moment)?;
    write_f32(writer, tail.p_factor)?;
    write_u8_array(writer, &tail.padding_1)?;
    write_u8_array(writer, &tail.phase_name)?;
    write_u8_array(writer, &tail.padding_2)
}

fn write_heat_capacity<W: Write>(
    writer: &mut W,
    chunk: &RawHeatCapacityChunk,
) -> Result<(), SerializeError> {
    write_common_header(writer, &chunk.header)?;
    write_f64(writer, chunk.enthalpy)?;
    write_f64(writer, chunk.entropy)?;
    write_i32(writer, chunk.phase_id_raw)?;
    write_u8_array(writer, &chunk.unknown_1)?;
    write_f64(writer, chunk.temperature_min)?;
    write_f64(writer, chunk.temperature_max)?;
    write_f64_array(writer, &chunk.coefficients)?;
    write_f64_array(writer, &chunk.powers)?;
    write_u8_array(writer, &chunk.padding_remaining)
}

fn write_comment<W: Write>(writer: &mut W, chunk: &RawCommentChunk) -> Result<(), SerializeError> {
    write_common_header(writer, &chunk.header)?;
    write_u8_array(writer, &chunk.comment)?;
    write_u8_array(writer, &chunk.padding_remaining)
}

fn write_kappa<W: Write>(writer: &mut W, chunk: &RawKappaChunk) -> Result<(), SerializeError> {
    write_common_header(writer, &chunk.header)?;
    write_f64(writer, chunk.temperature_min)?;
    write_f64(writer, chunk.temperature_max)?;
    write_i32(writer, chunk.phase_id_raw)?;
    write_u8_array(writer, &chunk.unknown_1)?;
    write_f64_array(writer, &chunk.f1_temperature_coefficients)?;
    write_f32_array(writer, &chunk.f1_temperature_powers)?;
    write_f64_array(writer, &chunk.f2_pressure_coefficients)?;
    write_f32_array(writer, &chunk.f2_pressure_powers)?;
    write_f64_array(writer, &chunk.f3_temperature_coefficients)?;
    write_f32_array(writer, &chunk.f3_temperature_powers)?;
    write_u8_array(writer, &chunk.padding_remaining)
}

fn write_u8<W: Write>(writer: &mut W, value: u8) -> Result<(), SerializeError> {
    writer.write_all(&[value]).map_err(SerializeError::from)
}

fn write_i8<W: Write>(writer: &mut W, value: i8) -> Result<(), SerializeError> {
    write_u8(writer, value as u8)
}

fn write_i32<W: Write>(writer: &mut W, value: i32) -> Result<(), SerializeError> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(SerializeError::from)
}

fn write_u32<W: Write>(writer: &mut W, value: u32) -> Result<(), SerializeError> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(SerializeError::from)
}

fn write_f32<W: Write>(writer: &mut W, value: f32) -> Result<(), SerializeError> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(SerializeError::from)
}

fn write_f64<W: Write>(writer: &mut W, value: f64) -> Result<(), SerializeError> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(SerializeError::from)
}

fn write_u8_array<W: Write, const N: usize>(
    writer: &mut W,
    values: &[u8; N],
) -> Result<(), SerializeError> {
    writer.write_all(values).map_err(SerializeError::from)
}

fn write_u16_array<W: Write, const N: usize>(
    writer: &mut W,
    values: &[u16; N],
) -> Result<(), SerializeError> {
    for value in values {
        writer
            .write_all(&value.to_le_bytes())
            .map_err(SerializeError::from)?;
    }
    Ok(())
}

fn write_f32_array<W: Write, const N: usize>(
    writer: &mut W,
    values: &[f32; N],
) -> Result<(), SerializeError> {
    for value in values {
        write_f32(writer, *value)?;
    }
    Ok(())
}

fn write_f64_array<W: Write, const N: usize>(
    writer: &mut W,
    values: &[f64; N],
) -> Result<(), SerializeError> {
    for value in values {
        write_f64(writer, *value)?;
    }
    Ok(())
}

const _: () = assert!(BODY_SIZE == 255);
