mod error;
mod lexer;
mod token;

pub use error::*;

#[cfg(test)]
mod tests {
    use strum::{Display, EnumIter};

    use crate::{
        lexer::{DefaultOps, Lexer},
        token::{Delimited, Operator, TokenPos},
    };

    #[derive(Clone, PartialEq, EnumIter, Debug)]
    enum Delimiter {
        Str(String),
        Arr(String),
    }

    impl Delimited for Delimiter {
        fn delimiters(&self) -> Option<(String, String)> {
            Some(match self {
                Delimiter::Str(_) => ("\"".into(), "\"".into()),
                Delimiter::Arr(_) => ("[".into(), "]".into()),
            })
        }

        fn set(&mut self, input: String) {
            match self {
                Delimiter::Str(x) => *x = input,
                Delimiter::Arr(x) => *x = input,
            }
        }
    }

    fn print_next_token<O: Operator, D: Delimited>(lexer: &mut Lexer<O, D>) {
        let res = lexer.next();
        match res {
            Ok(t) => println!("{:?}", t.token),
            Err(e) => println!("{e}"),
        }
    }

    #[test]
    fn lexer_default() {
        let mut lexer = Lexer::<DefaultOps, Delimiter>::new();

        println!("{lexer:#?}");

        lexer.set_text("name + 2 * 10.14 - asd[my array typeasd");
        print_next_token(&mut lexer);
        print_next_token(&mut lexer);
        print_next_token(&mut lexer);
        print_next_token(&mut lexer);
        print_next_token(&mut lexer);
        print_next_token(&mut lexer);
        print_next_token(&mut lexer);
        print_next_token(&mut lexer);
        print_next_token(&mut lexer);
        print_next_token(&mut lexer);
    }
}
