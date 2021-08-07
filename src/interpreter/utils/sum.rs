#[derive(PartialEq, Eq, Debug)]
pub enum SumTypes {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
}

pub trait Sum {
    fn get_sum_of(&self, file: &String, sum: SumTypes) -> String;
    fn read_bytes_of_file<'a>(&self, path: &String) -> &'a [u8];
}
