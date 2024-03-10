# Specification

evi is a vi clone editor written in Rust. It is a simple and lightweight text editor that is easy to use and has a small memory footprint. It is a work in progress and is not yet feature complete.

## Features

- [ ] Basic vi commands
- [ ] Ex commands
- [ ] Search and replace
- [ ] Unicode support
- [ ] Syntax highlighting

## Requirement

- テキストファイルを開いて画面に表示する。
- 画面上のカーソルを操作し、開いたテキストファイルの任意の位置を画面に表示する。
- テキストファイルの内容を変更する。
- ファイルの保存。
- ファイルの新規作成。
- ファイルの読み込み。
- ファイルの削除。
- 行番号表示。
- カーソル移動（上下左右、単語単位、行頭・行末への移動）。
- テキストの挿入（インサートモード）。
- テキストの削除（カーソル位置の文字、行の削除）。
- テキストのコピー（ビジュアルモードでの選択範囲）。
- テキストの貼り付け。
- テキストの検索（前方、後方）。
- テキストの置換（指定範囲内、全ファイル内）。
- 取り消し（Undo）。
- やり直し（Redo）。
- 自動インデント（新しい行の開始時に前行と同じインデントを適用）。
- 折り返し表示（長い行を画面幅に合わせて表示）。
- シンタックスハイライト（オプション、言語に応じた色分け）。
- 設定ファイルによる挙動のカスタマイズ。

