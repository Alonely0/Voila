use byte_unit::ByteUnit;

pub trait ByteConversion {
    fn convert(&self, from: u128, to: ByteUnit) -> f64;
}
