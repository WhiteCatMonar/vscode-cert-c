//! EXP44-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;
use crate::rules::common::{find_matching, find_side_effect};

/// EXP44-Cの診断を返す。
///
/// `sizeof`のオペランドに副作用を伴う演算子が含まれる場合に診断する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// EXP44-Cに関する診断一覧。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if token.text != "sizeof" {
            continue;
        }

        if tokens.get(index + 1).is_some_and(|next| next.text == "(") {
            let Some(close_index) = find_matching(tokens, index + 1, "(", ")") else {
                continue;
            };
            if let Some(side_effect) = find_side_effect(&tokens[index + 2..close_index]) {
                diagnostics.push(Diagnostic::new(
                    source,
                    side_effect.span,
                    "EXP44-C",
                    "sizeof 演算子のオペランドは副作用を持たせない",
                ));
            }
        } else if let Some(operand) = tokens.get(index + 1) {
            if find_side_effect(std::slice::from_ref(operand)).is_some() {
                diagnostics.push(Diagnostic::new(
                    source,
                    operand.span,
                    "EXP44-C",
                    "sizeof 演算子のオペランドは副作用を持たせない",
                ));
            }
        }
    }

    diagnostics
}
