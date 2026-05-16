//! ルール実装間で共有する補助関数。

use crate::lexer::Token;

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
