use super::Args;
use super::Literal;

pub trait Functions {
    // Parser returns the args of a function as a vector of vector of Literals,
    // because an argument might have Text & Variables. Each vector inside the
    // super-vector is a function argument (those separated by ","), and the
    // Literals inside those vectors must be merged into a unique Literal,
    // creating a vector of literals, being each one a function argument
    // for making something that a function can deal with more easily.
    // Then, the best is to create directly a vector of strings, because
    // we do not care anymore about the type of the literal.
    fn supervec_literals_to_args(&self, supervec: Vec<Vec<Literal>>) -> Args;

    // Functions definitions
    fn r#print(&self, args: Args);
    fn r#create(&self, args: Args);
    fn r#mkdir(&self, args: Args);
    fn r#delete(&self, args: Args);
    fn r#move(&self, args: Args);
    fn r#copy(&self, args: Args);
    fn r#shell(&self, args: Args);
}
