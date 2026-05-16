//! ソース全体の解析フローをまとめるモジュール。

use crate::diagnostic::Diagnostic;
use crate::lexer::tokenize;
use crate::rules;

/// 1つのCソース文字列を解析し、現在有効な全ルールの診断を返す。
///
/// # 引数
///
/// - `source`: 解析対象のCソースコード。
///
/// # 戻り値
///
/// 検出されたCERT-C診断の一覧。
pub fn analyze(source: &str) -> Vec<Diagnostic> {
    let tokens = tokenize(source);
    let mut diagnostics = Vec::new();

    diagnostics.extend(rules::dcl37_c::check(source, &tokens));
    diagnostics.extend(rules::exp45_c::check(source, &tokens));
    diagnostics.extend(rules::int33_c::check(source, &tokens));
    diagnostics.extend(rules::exp31_c::check(source, &tokens));
    diagnostics.extend(rules::pre31_c::check(source, &tokens));

    diagnostics
}
