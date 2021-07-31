pub trait Exceptions {
    fn raise_error(&self, err_type: &str, msg: String);
}