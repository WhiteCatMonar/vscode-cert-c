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

        for (condition_index, condition_token) in
            tokens[open_index + 1..close_index].iter().enumerate()
        {
            if is_assignment_operator(&condition_token.text) {
                let assignment_index = open_index + 1 + condition_index;
                if !is_parenthesized_assignment_expression(
                    tokens,
                    open_index,
                    close_index,
                    assignment_index,
                ) && !is_before_top_level_comma(tokens, close_index, assignment_index)
                {
                    diagnostics.push(Diagnostic::new(
                        source,
                        condition_token.span,
                        "EXP45-C",
                        "選択文に対して代入を行わない",
                    ));
                }
            }
        }
    }

    diagnostics
}

/// 代入演算子の後ろに制御式直下のコンマ演算子があるかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `condition_close`: 制御式全体を囲む閉じ括弧の位置。
/// - `assignment_index`: 判定対象の代入演算子位置。
///
/// # 戻り値
///
/// `while (x = y, p == q)`のように、代入式がコンマ式の途中要素である場合は`true`。
fn is_before_top_level_comma(
    tokens: &[Token],
    condition_close: usize,
    assignment_index: usize,
) -> bool {
    let mut depth = 0usize;

    for token in &tokens[assignment_index + 1..condition_close] {
        match token.text.as_str() {
            "(" => depth += 1,
            ")" if depth > 0 => depth -= 1,
            "," if depth == 0 => return true,
            _ => {}
        }
    }

    false
}

/// 代入式が明示的な括弧で囲まれているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `condition_open`: 制御式全体を囲む開き括弧の位置。
/// - `condition_close`: 制御式全体を囲む閉じ括弧の位置。
/// - `assignment_index`: 判定対象の代入演算子位置。
///
/// # 戻り値
///
/// `if ((a = b) != 0)`のように、代入式を括弧に入れたうえで比較している場合は`true`。
fn is_parenthesized_assignment_expression(
    tokens: &[Token],
    condition_open: usize,
    condition_close: usize,
    assignment_index: usize,
) -> bool {
    for open_index in condition_open + 1..assignment_index {
        if tokens[open_index].text != "(" {
            continue;
        }

        let Some(close_index) = find_matching(tokens, open_index, "(", ")") else {
            continue;
        };

        if close_index < condition_close
            && assignment_index < close_index
            && is_followed_by_comparison_operator(tokens, close_index)
        {
            return true;
        }
    }

    false
}

/// 括弧で囲まれた代入式の直後が比較演算子かを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `close_index`: 代入式を囲む閉じ括弧の位置。
///
/// # 戻り値
///
/// 直後が比較演算子の場合は`true`。
fn is_followed_by_comparison_operator(tokens: &[Token], close_index: usize) -> bool {
    tokens
        .get(close_index + 1)
        .is_some_and(|token| matches!(token.text.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">="))
}
