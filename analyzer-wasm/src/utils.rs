//! 汎用的な補助関数。

/// バイトオフセットを1始まりの行番号・列番号へ変換する。
///
/// # 引数
///
/// - `source`: 位置を計算する元ソースコード。
/// - `offset`: 変換対象のバイトオフセット。
///
/// # 戻り値
///
/// 1始まりの`(行番号, 列番号)`。
pub fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for (index, character) in source.char_indices() {
        if index >= offset {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}
