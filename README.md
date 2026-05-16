# vscode-cert-c

- [English version](README-en.md)

## 概要

VSCode上でCERT-C指向のチェックを実行するための拡張機能プロトタイプです。

Rust/WASMで実装した解析コアをTypeScriptから呼び出し、検出した診断をVSCodeのProblemsへ表示します。
現在は初期検証段階であり、C99を対象にCERT-Cルール対応を段階的に拡充していきます。

## 主な機能

- CソースファイルとヘッダファイルのCERT-C指向チェック
- VSCode Problemsへの診断表示
- ワークスペース全体の手動チェック
- 編集中または保存時の対象ファイル自動再解析
- Rust/WASM解析コアによる外部解析ツール非依存の検出

## 使い方

コマンドパレットから以下を実行します。

- `CERT-C: ワークスペースをチェック`
- `CERT-C: 診断をクリア`

既定では、`.c`ファイルと`.h`ファイルが解析対象です。

## 設定

```json
{
  "certC.fileGlobs": ["**/*.c", "**/*.h"],
  "certC.excludeGlobs": ["**/.git/**", "**/.svn/**", "**/.hg/**", "**/build/**", "**/cmake-build-*/**", "**/Debug/**", "**/Release/**", "**/dist/**", "**/vendor/**", "**/third_party/**"],
  "certC.analyzeOnChange": true,
  "certC.analyzeOnSave": true,
  "certC.analyzeDebounceMs": 500
}
```

## ドキュメント

- [アーキテクチャ概要](docs/ja/architecture-overview.md)
- [開発ガイド](docs/ja/development-guide.md)

## ライセンス

このプロジェクトはMIT Licenseで公開します。
