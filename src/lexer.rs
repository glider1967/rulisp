#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Integer(i64),
    Symbol(String),
    LParen,
    RParen,
    Quote,
}

pub fn lex(program: &str) -> Vec<Token> {
    let prog2 = program
        .replace("(", " ( ")
        .replace(")", " ) ")
        .replace("\'", " \' ");
    let words = prog2.split_whitespace();

    let mut tokens: Vec<Token> = vec![];
    for word in words {
        match word {
            "(" => tokens.push(Token::LParen),
            ")" => tokens.push(Token::RParen),
            "\'" => tokens.push(Token::Quote),
            _ => {
                let i = word.parse::<i64>();
                if i.is_ok() {
                    tokens.push(Token::Integer(i.unwrap()));
                } else {
                    tokens.push(Token::Symbol(word.to_string()));
                }
            }
        }
    }

    tokens
}
