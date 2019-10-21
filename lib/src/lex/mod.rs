mod token;

pub use token::Token;

pub fn tokenize<S: Into<String>>(source: S) -> Result<Vec<Token>, String> {
    use Token::*;

    let mut tokens: Vec<Token> = vec![BlockOpen, BlockOpen];
    let source = source.into();
    let mut chars = source.chars().peekable();

    let mut read_until_depths = Vec::new();
    let mut bracket_depth = 0;

    // I want variadic push
    macro_rules! token_push_internal {
        ( $(,)* ) => {};
        (BlockOpen, $($tail:tt)*) => {
            bracket_depth += 1;
            tokens.push(BlockOpen);
            token_push!($($tail)*);
        };
        (BlockClose, $($tail:tt)*) => {
            if let Some(depth) = read_until_depths.last() {
                //println!("There's a depth to read until");
                if *depth == bracket_depth {
                    //println!("It's the depth we're reading until the end of");
                    read_until_depths.pop();
                    tokens.push(BlockClose);
                }
            }
            tokens.push(BlockClose);

            #[allow(unused_assignments)]
            bracket_depth -= 1;

            token_push!($($tail)*);
        };
        ($name:expr, $($tail:tt)*) => {
            tokens.push($name);
            token_push!($($tail)*);
        };
    }
    macro_rules! token_push {
        ( $($tail:tt)* ) => {
            {
                token_push_internal!($($tail)*,);
            }
        };
    }

    while let Some(c) = chars.next() {
        match c {
            '<' => match chars.peek() {
                Some('-') => {
                    chars.next();
                    token_push!(StorageArrow);

                    read_until_depths.push(bracket_depth);
                    tokens.push(BlockOpen);
                }
                _ => token_push!(LessThan),
            },

            '\n' => {
                token_push!(BlockClose, BlockOpen);
            }
            '{' => token_push!(BlockOpen),
            '}' => token_push!(BlockClose),
            '(' => token_push!(ArgsOpen),
            ')' => token_push!(ArgsClose),
            '+' | '-' | '*' | '/' => token_push!(BinaryOperation(c.to_string())),
            c => {
                if c.is_whitespace() {
                    // do nothing
                } else if c.is_alphanumeric() || c == '"' {
                    let mut name = c.to_string();

                    if c != '"' {
                        while let Some(fc) = chars.peek() {
                            if fc.is_alphanumeric() {
                                name.push(chars.next().unwrap())
                            } else {
                                break;
                            }
                        }
                    } else {
                        while let Some(fc) = chars.peek() {
                            if *fc != '"' {
                                name.push(chars.next().unwrap())
                            } else {
                                break;
                            }
                        }
                    }

                    if c == '"' {
                        // pushing it if it's a string literal.
                        if !(chars.next() == Some('"')) {
                            return Err("Unfinished string literal".to_string());
                        }
                        name.remove(0);
                        token_push!(StringLiteral(name));
                    } else if let Ok(n) = name.parse() {
                        // numbers
                        token_push!(Number(n));
                    } else if name == "MOD" {
                        // special case for the MOD operator
                        token_push!(BinaryOperation(name.to_string()))
                    } else {
                        // then it's gotta be an identifier.
                        token_push!(Identifier(name));
                    }
                } else {
                    eprintln!("ignoring {}", c);
                }
            }
        }
    }

    token_push!(BlockClose, BlockClose);
    Ok(tokens)
}

#[test]
fn test_tokenize() {
    use Token::*;

    assert_eq!(
        tokenize("s <- 3").unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    );

    assert_eq!(
        tokenize("s<-3").unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    );

    assert_eq!(
        tokenize("s <- 3 + 2").unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                        BinaryOperation("+".to_string()),
                        Number(2.0),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    );
    assert_eq!(
        tokenize("s<-3+2").unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                        BinaryOperation("+".to_string()),
                        Number(2.0),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    );

    assert_eq!(
        tokenize(
            "\
            s <- 3
            DISPLAY(s)\
        "
        )
        .unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("DISPLAY".to_string()), ArgsOpen,
                        Identifier("s".to_string()),
                    ArgsClose,
                BlockClose,
            BlockClose,
        ]
    );

    assert_eq!(
        tokenize(
            "\
            s<-3
            DISPLAY(s)\
        "
        )
        .unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("DISPLAY".to_string()), ArgsOpen,
                        Identifier("s".to_string()),
                    ArgsClose,
                BlockClose,
            BlockClose,
        ]
    );

    assert_eq!(
        tokenize(
            "\
            s <- 3
            l <- 4
            a <- 1
            s <- a + 5
            l <- a
            a <- a + 3
            DISPLAY(s)
            DISPLAY(l)
            DISPLAY(a)\
        "
        )
        .unwrap(),
        #[rustfmt::skip]
        vec![
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("l".to_string()), StorageArrow, BlockOpen,
                        Number(4.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("a".to_string()), StorageArrow, BlockOpen,
                        Number(1.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Identifier("a".to_string()), BinaryOperation("+".to_string()), Number(5.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("l".to_string()), StorageArrow, BlockOpen,
                        Identifier("a".to_string()),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("a".to_string()), StorageArrow, BlockOpen,
                        Identifier("a".to_string()), BinaryOperation("+".to_string()), Number(3.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("DISPLAY".to_string()), ArgsOpen,
                        Identifier("s".to_string()),
                    ArgsClose,
                BlockClose,
                BlockOpen,
                    Identifier("DISPLAY".to_string()), ArgsOpen,
                        Identifier("l".to_string()),
                    ArgsClose,
                BlockClose,
                BlockOpen,
                    Identifier("DISPLAY".to_string()), ArgsOpen,
                        Identifier("a".to_string()),
                    ArgsClose,
                BlockClose,
            BlockClose,
        ]
    )
}
