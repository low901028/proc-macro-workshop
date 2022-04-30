// This test looks for a function-like macro with the right name to exist. For
// now the test doesn't require any specific code to be generated by the macro,
// so returning an empty TokenStream should be sufficient.
//
// Before moving on to the next test, you'll want some code in your
// implementation to handle parsing the first few tokens of input. The macro
// should expect the input to contain a syn::Ident, Token![in], syn::LitInt,
// Token![..], syn::LitInt.
//
// It is also possible to implement this project without using Syn if you'd
// like, though you will end up writing more code, more tedious code, and more
// explicit error handling than when using Syn as a parsing library.
//
//
// Resources:
//
//   - Parsing in Syn:
//     https://docs.rs/syn/0.15/syn/parse/index.html
//
//   - An example of a function-like procedural macro implemented using Syn:
//     https://github.com/dtolnay/syn/tree/master/examples/lazy-static

use seq::seq;

seq!(N in 0..8 {
    // nothing
});

fn main() {}
