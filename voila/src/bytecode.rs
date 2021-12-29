use crate::ast::Script;

pub type VoilaByteCode = Vec<u8>;

impl<'source> From<Script<'source>> for VoilaByteCode {
    fn from(code: Script<'source>) -> Self {
        bincode::serialize(&code).unwrap()
    }
}

impl<'source> From<VoilaByteCode> for Script<'source> {
    fn from(s: VoilaByteCode) -> Self {
        bincode::deserialize(&s[..]).unwrap()
    }
}
