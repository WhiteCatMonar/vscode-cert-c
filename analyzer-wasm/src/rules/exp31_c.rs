//! EXP31-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;
use crate::rules::common::{find_matching, find_next, find_side_effect};

/// EXP31-Cの診断を返す。
///
/// `assert(...)`の引数内にインクリメント、デクリメント、代入がある場合に診断する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// EXP31-Cに関する診断一覧。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if token.text != "assert" {
            continue;
        }

        let Some(open_index) = find_next(tokens, index + 1, "(") else {
            continue;
        };
        let Some(close_index) = find_matching(tokens, open_index, "(", ")") else {
            continue;
        };

        if let Some(side_effect) = find_side_effect(&tokens[open_index + 1..close_index]) {
            diagnostics.push(Diagnostic::new(
                source,
                side_effect.span,
                "EXP31-C",
                "assertの引数に副作用を含めないでください。",
            ));
        }
    }

    diagnostics
}
