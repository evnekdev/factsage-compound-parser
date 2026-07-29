use crate::error::ParseError;

pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
    chunk_index: usize,
    base_offset: usize,
    record_type: &'static str,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(
        bytes: &'a [u8],
        chunk_index: usize,
        base_offset: usize,
        record_type: &'static str,
    ) -> Self {
        Self {
            bytes,
            position: 0,
            chunk_index,
            base_offset,
            record_type,
        }
    }

    pub(crate) fn offset(&self) -> usize {
        self.base_offset + self.position
    }

    pub(crate) fn read_u8(&mut self, field: &'static str) -> Result<u8, ParseError> {
        Ok(self.read_bytes::<1>(field)?[0])
    }

    pub(crate) fn read_i8(&mut self, field: &'static str) -> Result<i8, ParseError> {
        Ok(self.read_u8(field)? as i8)
    }

    pub(crate) fn read_u16(&mut self, field: &'static str) -> Result<u16, ParseError> {
        Ok(u16::from_le_bytes(self.read_bytes::<2>(field)?))
    }

    pub(crate) fn read_i32(&mut self, field: &'static str) -> Result<i32, ParseError> {
        Ok(i32::from_le_bytes(self.read_bytes::<4>(field)?))
    }

    pub(crate) fn read_u32(&mut self, field: &'static str) -> Result<u32, ParseError> {
        Ok(u32::from_le_bytes(self.read_bytes::<4>(field)?))
    }

    pub(crate) fn read_f32(&mut self, field: &'static str) -> Result<f32, ParseError> {
        Ok(f32::from_le_bytes(self.read_bytes::<4>(field)?))
    }

    pub(crate) fn read_f64(&mut self, field: &'static str) -> Result<f64, ParseError> {
        Ok(f64::from_le_bytes(self.read_bytes::<8>(field)?))
    }

    pub(crate) fn read_bytes<const N: usize>(
        &mut self,
        field: &'static str,
    ) -> Result<[u8; N], ParseError> {
        let remaining = self.bytes.len().saturating_sub(self.position);
        if remaining < N {
            return Err(ParseError::FieldBoundary {
                chunk_index: self.chunk_index,
                byte_offset: self.offset(),
                record_type: self.record_type,
                field,
                requested: N,
                remaining,
            });
        }

        let end = self.position + N;
        let mut result = [0_u8; N];
        result.copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        Ok(result)
    }

    pub(crate) fn read_u8_array<const N: usize>(
        &mut self,
        field: &'static str,
    ) -> Result<[u8; N], ParseError> {
        self.read_bytes(field)
    }

    pub(crate) fn read_u16_array<const N: usize>(
        &mut self,
        field: &'static str,
    ) -> Result<[u16; N], ParseError> {
        let mut result = [0_u16; N];
        for value in &mut result {
            *value = self.read_u16(field)?;
        }
        Ok(result)
    }

    pub(crate) fn read_f32_array<const N: usize>(
        &mut self,
        field: &'static str,
    ) -> Result<[f32; N], ParseError> {
        let mut result = [0_f32; N];
        for value in &mut result {
            *value = self.read_f32(field)?;
        }
        Ok(result)
    }

    pub(crate) fn read_f64_array<const N: usize>(
        &mut self,
        field: &'static str,
    ) -> Result<[f64; N], ParseError> {
        let mut result = [0_f64; N];
        for value in &mut result {
            *value = self.read_f64(field)?;
        }
        Ok(result)
    }

    pub(crate) fn finish(&self) -> Result<(), ParseError> {
        if self.position != self.bytes.len() {
            return Err(ParseError::FieldBoundary {
                chunk_index: self.chunk_index,
                byte_offset: self.offset(),
                record_type: self.record_type,
                field: "body_end",
                requested: 0,
                remaining: self.bytes.len() - self.position,
            });
        }
        Ok(())
    }
}
