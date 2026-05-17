//! ルール実装間で共有する補助関数。

use crate::lexer::Token;

/// 関数または関数形式マクロの呼び出し範囲を表す。
pub struct CallArguments {
    pub open_index: usize,
    pub close_index: usize,
    pub arguments: Vec<Vec<Token>>,
}

/// トークン列内に副作用を持つ演算子があれば、そのトークンを返す。
///
/// # 引数
///
/// - `tokens`: 副作用の有無を調べるトークン列。
///
/// # 戻り値
///
/// 副作用を持つ演算子トークン。見つからない場合は`None`。
pub fn find_side_effect(tokens: &[Token]) -> Option<&Token> {
    tokens.iter().find(|token| {
        token.text == "++" || token.text == "--" || is_assignment_operator(&token.text)
    })
}

/// 指定位置以降で、指定テキストを持つ最初のトークン位置を返す。
///
/// # 引数
///
/// - `tokens`: 検索対象のトークン列。
/// - `start`: 検索を開始するトークン位置。
/// - `text`: 検索するトークン文字列。
///
/// # 戻り値
///
/// 見つかったトークン位置。見つからない場合は`None`。
pub fn find_next(tokens: &[Token], start: usize, text: &str) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, token)| (token.text == text).then_some(index))
}

/// 指定位置の識別子に続く関数呼び出し引数を返す。
///
/// # 引数
///
/// - `tokens`: 検索対象のトークン列。
/// - `name_index`: 関数名または関数形式マクロ名のトークン位置。
///
/// # 戻り値
///
/// 呼び出し括弧と分割済み実引数。呼び出し形式でない場合は`None`。
pub fn find_call_arguments(tokens: &[Token], name_index: usize) -> Option<CallArguments> {
    let open_index = find_next(tokens, name_index + 1, "(")?;
    if tokens[name_index + 1..open_index]
        .iter()
        .any(|token| token.text != " ")
    {
        return None;
    }
    let close_index = find_matching(tokens, open_index, "(", ")")?;
    let arguments = split_arguments(&tokens[open_index + 1..close_index]);
    Some(CallArguments {
        open_index,
        close_index,
        arguments,
    })
}

/// 対応する閉じトークンの位置を返す。
///
/// `open_index`は開きトークン位置を指定する。ネストした括弧にも対応する。
///
/// # 引数
///
/// - `tokens`: 検索対象のトークン列。
/// - `open_index`: 開きトークンの位置。
/// - `open`: 開きトークン文字列。
/// - `close`: 閉じトークン文字列。
///
/// # 戻り値
///
/// 対応する閉じトークン位置。対応が見つからない場合は`None`。
pub fn find_matching(
    tokens: &[Token],
    open_index: usize,
    open: &str,
    close: &str,
) -> Option<usize> {
    let mut depth = 0;

    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        if token.text == open {
            depth += 1;
        } else if token.text == close {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }

    None
}

/// 副作用を伴う代入演算子かを返す。
///
/// # 引数
///
/// - `value`: 判定対象のトークン文字列。
///
/// # 戻り値
///
/// 代入演算子として扱う場合は`true`。
pub fn is_assignment_operator(value: &str) -> bool {
    matches!(value, "=" | "+=" | "-=" | "*=" | "/=" | "%=")
}

/// 指定位置の識別子が宣言内の名前として現れている可能性を返す。
///
/// # 引数
///
/// - `tokens`: 判定対象のトークン列。
/// - `name_index`: 識別子のトークン位置。
///
/// # 戻り値
///
/// 宣言キーワードの直後にある名前として扱う場合は`true`。
///
/// TODO: 宣言Parserで関数宣言と式中の呼び出しを正確に区別する。
pub fn is_declared_name(tokens: &[Token], name_index: usize) -> bool {
    name_index > 0 && is_declaration_keyword(&tokens[name_index - 1].text)
}

/// 宣言キーワードとして扱うトークンかを返す。
///
/// # 引数
///
/// - `value`: 判定対象のトークン文字列。
///
/// # 戻り値
///
/// 宣言キーワードとして扱う場合は`true`。
pub fn is_declaration_keyword(value: &str) -> bool {
    matches!(
        value,
        "char"
            | "double"
            | "enum"
            | "extern"
            | "float"
            | "int"
            | "long"
            | "short"
            | "signed"
            | "static"
            | "struct"
            | "union"
            | "unsigned"
            | "void"
    )
}

/// 関数呼び出しやマクロ呼び出しの実引数トークン列をカンマ区切りで分割する。
///
/// # 引数
///
/// - `tokens`: 実引数部分のトークン列。
///
/// # 戻り値
///
/// 実引数ごとのトークン列。
pub fn split_arguments(tokens: &[Token]) -> Vec<Vec<Token>> {
    let mut arguments = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0;

    for token in tokens {
        if token.text == "(" {
            depth += 1;
        } else if token.text == ")" && depth > 0 {
            depth -= 1;
        }

        if token.text == "," && depth == 0 {
            arguments.push(current);
            current = Vec::new();
        } else {
            current.push(token.clone());
        }
    }

    if !current.is_empty() {
        arguments.push(current);
    }

    arguments
}
