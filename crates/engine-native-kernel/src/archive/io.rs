use super::*;

// ---- 写入器 ----

pub(super) struct W {
    pub(super) buf: Vec<u8>,
}

impl W {
    pub(super) fn new() -> Self {
        Self { buf: Vec::new() }
    }
    pub(super) fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    pub(super) fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    pub(super) fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    pub(super) fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    pub(super) fn i32(&mut self, v: i32) {
        self.u32(v as u32);
    }
    pub(super) fn boolean(&mut self, v: bool) {
        self.u8(v as u8);
    }
    pub(super) fn opt_u16(&mut self, v: Option<u16>) {
        match v {
            Some(x) => {
                self.u8(1);
                self.u16(x);
            }
            None => self.u8(0),
        }
    }
    pub(super) fn opt_u8(&mut self, v: Option<u8>) {
        match v {
            Some(x) => {
                self.u8(1);
                self.u8(x);
            }
            None => self.u8(0),
        }
    }
    pub(super) fn len(&mut self, n: usize) {
        self.u64(n as u64);
    }
    pub(super) fn bytes(&mut self, b: &[u8]) {
        self.len(b.len());
        self.buf.extend_from_slice(b);
    }
    pub(super) fn str(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }
}

// ---- 读取器 ----

pub(super) struct R<'a> {
    pub(super) data: &'a [u8],
    pub(super) pos: usize,
}

impl<'a> R<'a> {
    pub(super) fn take(&mut self, n: usize) -> Result<&'a [u8], ArchiveError> {
        let end = self.pos + n;
        if end > self.data.len() {
            return Err(ArchiveError::UnexpectedEof);
        }
        let out = &self.data[self.pos..end];
        self.pos = end;
        Ok(out)
    }
    pub(super) fn u8(&mut self) -> Result<u8, ArchiveError> {
        Ok(self.take(1)?[0])
    }
    pub(super) fn u16(&mut self) -> Result<u16, ArchiveError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub(super) fn u32(&mut self) -> Result<u32, ArchiveError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub(super) fn u64(&mut self) -> Result<u64, ArchiveError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub(super) fn i32(&mut self) -> Result<i32, ArchiveError> {
        Ok(self.u32()? as i32)
    }
    pub(super) fn boolean(&mut self) -> Result<bool, ArchiveError> {
        Ok(self.u8()? != 0)
    }
    pub(super) fn opt_u16(&mut self) -> Result<Option<u16>, ArchiveError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.u16()?)),
            t => Err(ArchiveError::BadTag("opt_u16", t)),
        }
    }
    pub(super) fn opt_u8(&mut self) -> Result<Option<u8>, ArchiveError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.u8()?)),
            t => Err(ArchiveError::BadTag("opt", t)),
        }
    }
    pub(super) fn len(&mut self) -> Result<usize, ArchiveError> {
        Ok(self.u64()? as usize)
    }
    pub(super) fn bytes(&mut self) -> Result<&'a [u8], ArchiveError> {
        let n = self.len()?;
        self.take(n)
    }
    pub(super) fn vec<T>(
        &mut self,
        mut f: impl FnMut(&mut R<'a>) -> Result<T, ArchiveError>,
    ) -> Result<Vec<T>, ArchiveError> {
        let n = self.len()?;
        let mut out = Vec::with_capacity(n.min(1024));
        for _ in 0..n {
            out.push(f(self)?);
        }
        Ok(out)
    }
}
