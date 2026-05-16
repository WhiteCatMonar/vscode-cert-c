//! EXP45-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;
use crate::rules::common::{find_matching, find_next, is_assignment_operator};

/// EXP45-Cの診断を返す。
///
/// `if`、`while`、`for`の制御式の中に単純な代入演算子がある場合に診断する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// EXP45-Cに関する診断一覧。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if !matches!(token.text.as_str(), "if" | "while" | "for") {
            continue;
        }

        let Some(open_index) = find_next(tokens, index + 1, "(") else {
            continue;
        };
        let Some(close_index) = find_matching(tokens, open_index, "(", ")") else {
            continue;
        };

        for condition_token in &tokens[open_index + 1..close_index] {
            if is_assignment_operator(&condition_token.text) {
                diagnostics.push(Diagnostic::new(
                    source,
                    condition_token.span,
                    "EXP45-C",
                    "選択文または反復文の制御式で代入を使用しないでください。",
                ));
            }
        }
    }

    diagnostics
}
