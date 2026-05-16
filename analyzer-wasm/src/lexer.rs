//! 軽量字句解析を扱うモジュール。

/// ソースコード上のバイト範囲。
#[derive(Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// 軽量字句解析で得られるトークン。
#[derive(Clone)]
pub struct Token {
    pub text: String,
    pub span: Span,
}

/// Cソースをコメント除去済みの軽量トークン列へ変換する。
///
/// 識別子、リテラル、演算子、括弧をトークンとして扱う。
///
/// # 引数
///
/// - `source`: トークン化するCソースコード。
///
/// # 戻り値
///
/// ソース上の位置情報を持つトークン列。
///
/// TODO: C99の字句仕様に沿った完全なLexerへ拡張する。
pub fn tokenize(source: &str) -> Vec<Token> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        let current = bytes[index];

        if current.is_ascii_whitespace() {
            index += 1;
            continue;
        }

        // コメント内のコード片を誤検出しないよう、行コメントとブロックコメントは捨てる。
        if current == b'/' && index + 1 < bytes.len() && bytes[index + 1] == b'/' {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }

        if current == b'/' && index + 1 < bytes.len() && bytes[index + 1] == b'*' {
            index += 2;
            while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            continue;
        }

        if current == b'"' || current == b'\'' {
            let quote = current;
            let start = index;
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                    continue;
                }
                if bytes[index] == quote {
                    index += 1;
                    break;
                }
                index += 1;
            }
            tokens.push(Token {
                text: source[start..index].to_string(),
                span: Span { start, end: index },
            });
            continue;
        }

        if is_identifier_start(current) {
            let start = index;
            index += 1;
            while index < bytes.len() && is_identifier_part(bytes[index]) {
                index += 1;
            }
            tokens.push(Token {
                text: source[start..index].to_string(),
                span: Span { start, end: index },
            });
            continue;
        }

        if current.is_ascii_digit() {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric()
                    || bytes[index] == b'.'
                    || bytes[index] == b'_')
            {
                index += 1;
            }
            tokens.push(Token {
                text: source[start..index].to_string(),
                span: Span { start, end: index },
            });
            continue;
        }

        if index + 1 < bytes.len() {
            let two = &source[index..index + 2];
            if matches!(
                two,
                "==" | "!="
                    | "<="
                    | ">="
                    | "++"
                    | "--"
                    | "&&"
                    | "||"
                    | "+="
                    | "-="
                    | "*="
                    | "/="
                    | "%="
            ) {
                tokens.push(Token {
                    text: two.to_string(),
                    span: Span {
                        start: index,
                        end: index + 2,
                    },
                });
                index += 2;
                continue;
            }
        }

        tokens.push(Token {
            text: source[index..index + 1].to_string(),
            span: Span {
                start: index,
                end: index + 1,
            },
        });
        index += 1;
    }

    tokens
}

/// C識別子の先頭文字として扱えるASCIIバイトかを返す。
///
/// # 引数
///
/// - `value`: 判定対象の1バイト文字。
///
/// # 戻り値
///
/// C識別子の先頭として扱う場合は`true`。
fn is_identifier_start(value: u8) -> bool {
    value.is_ascii_alphabetic() || value == b'_'
}

/// C識別子の2文字目以降として扱えるASCIIバイトかを返す。
///
/// # 引数
///
/// - `value`: 判定対象の1バイト文字。
///
/// # 戻り値
///
/// C識別子の2文字目以降として扱う場合は`true`。
fn is_identifier_part(value: u8) -> bool {
    value.is_ascii_alphanumeric() || value == b'_'
}
