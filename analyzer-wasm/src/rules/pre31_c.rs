//! PRE31-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, tokenize};
use crate::rules::common::{find_matching, find_next, find_side_effect};

/// PRE31-Cの診断を返す。
///
/// 関数形式マクロの置換リストで同じ仮引数が複数回現れる場合、その仮引数は安全でないと
/// みなし、呼び出し側の対応する実引数に副作用があれば診断する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// PRE31-Cに関する診断一覧。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let macros = find_unsafe_function_like_macros(source);

    for (macro_name, parameters) in macros {
        for (index, token) in tokens.iter().enumerate() {
            if token.text != macro_name {
                continue;
            }

            let Some(open_index) = find_next(tokens, index + 1, "(") else {
                continue;
            };
            let Some(close_index) = find_matching(tokens, open_index, "(", ")") else {
                continue;
            };

            let arguments = split_arguments(&tokens[open_index + 1..close_index]);
            for (argument_index, argument) in arguments.iter().enumerate() {
                if parameters.get(argument_index).copied().unwrap_or(false) {
                    if let Some(side_effect) = find_side_effect(argument) {
                        diagnostics.push(Diagnostic::new(
                            source,
                            side_effect.span,
                            "PRE31-C",
                            "安全でない関数形式マクロへ副作用を持つ引数を渡さないでください。",
                        ));
                    }
                }
            }
        }
    }

    diagnostics
}

/// 関数形式マクロを探し、仮引数ごとに「複数回評価され得るか」を返す。
///
/// # 引数
///
/// - `source`: マクロ定義を探すCソースコード。
///
/// # 戻り値
///
/// マクロ名と、仮引数ごとの複数回評価可能性を表す配列。
fn find_unsafe_function_like_macros(source: &str) -> Vec<(String, Vec<bool>)> {
    let mut macros = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("#define ") {
            continue;
        }

        let rest = trimmed.trim_start_matches("#define ").trim_start();
        let Some(name_end) = rest.find('(') else {
            continue;
        };
        let name = rest[..name_end].trim();
        if name.is_empty() || name.contains(char::is_whitespace) {
            continue;
        }

        let Some(parameter_end) = rest[name_end + 1..].find(')') else {
            continue;
        };
        let parameters_text = &rest[name_end + 1..name_end + 1 + parameter_end];
        let replacement = &rest[name_end + 2 + parameter_end..];
        let parameters: Vec<&str> = parameters_text
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect();
        let unsafe_parameters = parameters
            .iter()
            .map(|parameter| count_identifier_occurrences(replacement, parameter) > 1)
            .collect();

        macros.push((name.to_string(), unsafe_parameters));
    }

    macros
}

/// 指定した識別子がテキスト内にトークンとして現れる回数を返す。
///
/// # 引数
///
/// - `text`: 出現回数を調べるテキスト。
/// - `identifier`: 検索対象の識別子名。
///
/// # 戻り値
///
/// 識別子トークンとして現れた回数。
fn count_identifier_occurrences(text: &str, identifier: &str) -> usize {
    tokenize(text)
        .iter()
        .filter(|token| token.text == identifier)
        .count()
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
fn split_arguments(tokens: &[Token]) -> Vec<Vec<Token>> {
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
