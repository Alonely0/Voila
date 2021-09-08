use byte_unit::ByteUnit;

pub trait ByteConversion {
    fn convert_bytes(&self, from: u128, to: ByteUnit) -> f64;
}
