pub trait Exceptions {
    fn raise_parse_error(&self, err_type: &str, msg: String);
}