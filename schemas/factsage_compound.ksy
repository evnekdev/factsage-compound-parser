meta:
  id: factsage_compound
  title: FactSage Compound Database
  application: FactSage Compound module
  file-extension: cdb
  endian: le
  ks-version: 0.10
  license: MIT

doc: |
  Reverse-engineered layout of a FactSage Compound Database (.CDB).
  The file is a flat sequence of 256-byte chunks. Each chunk begins with
  a one-byte ID and has a 255-byte body. Unknown chunk IDs are retained
  as raw bodies because the body is size-bounded.

seq:
  - id: chunks
    type: chunk
    repeat: eos

enums:
  chunk_type:
    1: compound
    2: cp_1
    3: cp_4
    4: cp_2
    5: cp_3
    6: cp_5
    7: phase_ordinary
    8: phase_transition
    9: database_header
    10: comment
    11: kappa

types:
  chunk:
    doc: One fixed-size 256-byte database record.
    seq:
      - id: kind
        type: u1
        enum: chunk_type
      - id: body
        size: 255
        type:
          switch-on: kind
          cases:
            chunk_type::compound: compound_body
            chunk_type::cp_1: cp_body
            chunk_type::cp_4: cp_body
            chunk_type::cp_2: cp_body
            chunk_type::cp_3: cp_body
            chunk_type::cp_5: cp_body
            chunk_type::phase_ordinary: phase_ordinary_body
            chunk_type::phase_transition: phase_transition_body
            chunk_type::database_header: database_header_body
            chunk_type::comment: comment_body
            chunk_type::kappa: kappa_body

  database_header_body:
    seq:
      - id: padding_1
        type: u1
      - id: magic
        contents: [0x43, 0x4d, 0x50, 0x44]
      - id: padding_2
        size: 2
      - id: date_ole
        type: f8
        doc: OLE Automation date stored as a day-counting f64.
      - id: read_flag
        type: u1
      - id: unknown_1
        size: 11
      - id: comment
        type: str
        size: 80
        encoding: ASCII
        pad-right: 0
      - id: padding_3
        size: 136
      - id: unknown_2
        size: 12

  entry_header:
    doc: Shared 31-byte header used by all non-database chunk bodies.
    seq:
      - id: element_ids
        type: u1
        repeat: expr
        repeat-expr: 7
      - id: padding_coeff
        type: u1
      - id: element_coefficients
        type: u1
        repeat: expr
        repeat-expr: 7
      - id: charge_raw
        type: s1
      - id: entry_number
        type: u1
      - id: reference
        type: u2
        repeat: expr
        repeat-expr: 2
      - id: timestamp_ole
        type: f8
        doc: OLE Automation date stored as a day-counting f64.
      - id: unknown_1
        size: 2

  compound_body:
    seq:
      - id: header
        type: entry_header
      - id: compound_name
        type: str
        size: 40
        encoding: ASCII
        pad-right: 0
      - id: reserved_string_1
        type: str
        size: 40
        encoding: ASCII
        pad-right: 0
      - id: formula_name
        type: str
        size: 40
        encoding: ASCII
        pad-right: 0
      - id: unknown
        size: 4
      - id: unit_energy
        type: u4
      - id: unit_pressure
        type: u4
      - id: reserved_string_2
        type: str
        size: 12
        encoding: ASCII
        pad-right: 0
      - id: real_stoichiometric_coefficients
        type: f8
        repeat: expr
        repeat-expr: 7
      - id: padding_final
        size: 24

  phase_physical_tail:
    doc: Common 200-byte tail of ordinary and transition phase bodies.
    seq:
      - id: density_raw
        type: f8
      - id: thermal_expansion_coefficients
        type: f4
        repeat: expr
        repeat-expr: 4
      - id: compressibility_coefficients
        type: f4
        repeat: expr
        repeat-expr: 4
      - id: bulk_modulus_derivative_coefficients
        type: f4
        repeat: expr
        repeat-expr: 2
      - id: magnetic_temperature
        type: f4
      - id: magnetic_moment
        type: f4
      - id: p_factor
        type: f4
      - id: padding_1
        size: 20
      - id: phase_name
        type: str
        size: 40
        encoding: ASCII
        pad-right: 0
      - id: padding_2
        size: 80

  phase_ordinary_body:
    seq:
      - id: header
        type: entry_header
      - id: enthalpy
        type: f8
      - id: entropy
        type: f8
      - id: phase_id_raw_neg
        type: s4
      - id: phase_id_raw
        type: s4
      - id: physical
        type: phase_physical_tail

  phase_transition_body:
    seq:
      - id: header
        type: entry_header
      - id: transition_enthalpy
        type: f8
      - id: transition_temperature
        type: f8
      - id: parent_phase_id_raw
        type: s4
      - id: phase_id_raw
        type: s4
      - id: physical
        type: phase_physical_tail

  cp_body:
    seq:
      - id: header
        type: entry_header
      - id: enthalpy
        type: f8
      - id: entropy
        type: f8
      - id: phase_id_raw
        type: s4
      - id: unknown_1
        size: 4
      - id: temperature_min
        type: f8
      - id: temperature_max
        type: f8
      - id: coefficients
        type: f8
        repeat: expr
        repeat-expr: 8
      - id: powers
        type: f8
        repeat: expr
        repeat-expr: 8
      - id: padding_remaining
        size: 56

  comment_body:
    seq:
      - id: header
        type: entry_header
      - id: comment
        type: str
        size: 80
        encoding: ASCII
        pad-right: 0
      - id: padding_remaining
        size: 144

  kappa_body:
    seq:
      - id: header
        type: entry_header
      - id: temperature_min
        type: f8
      - id: temperature_max
        type: f8
      - id: phase_id_raw
        type: s4
      - id: unknown_1
        size: 4
      - id: f1_temperature_coefficients
        type: f8
        repeat: expr
        repeat-expr: 10
      - id: f1_temperature_powers
        type: f4
        repeat: expr
        repeat-expr: 8
      - id: f2_pressure_coefficients
        type: f8
        repeat: expr
        repeat-expr: 3
      - id: f2_pressure_powers
        type: f4
        repeat: expr
        repeat-expr: 2
      - id: f3_temperature_coefficients
        type: f8
        repeat: expr
        repeat-expr: 5
      - id: f3_temperature_powers
        type: f4
        repeat: expr
        repeat-expr: 3
      - id: padding_remaining
        size: 4
