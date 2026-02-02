# AquaVoice Alternative

AquaVoice を使って文字起こししていたのですが、無料プランだとレートリミットに引っかかってしまいました。
そこで、Gemini API KEY を使用して文字起こし可能な、Tauri製のアプリケーションを作成しました。

## 特徴

- バックグラウンド常時起動
- UI画面にてキーバインド設定可能
- Gemini のモデルと API KEY をUIより設定可能
- トレイアイコンに処理状態をアニメーション表示
- カスタマイズ可能な文字起こしプロンプト

## 必要な環境

### 開発者（ソースからビルドする場合）

- Node.js 18.0以降
- npm または yarn
- Rust 1.70以降
- macOS 12.0以降（現在macOS専用）

## インストール

### ビルド済みアプリを使用する場合

1. [Releases](https://github.com/kspace-trk/aqua-voice-alternative/releases)から最新の`.dmg`ファイルをダウンロード
2. `.dmg`ファイルを開き、アプリを`Applications`フォルダにドラッグ
3. アプリを起動（初回起動時にマイクとアクセシビリティの権限を要求されます）

### ソースからビルドする場合

```bash
# リポジトリをクローン
git clone https://github.com/kspace-trk/aqua-voice-alternative.git
cd aqua-voice-alternative

# 依存関係をインストール
npm install

# リリースビルドを作成
npm run build
```

ビルド成果物は以下に作成されます:
- `src-tauri/target/release/bundle/macos/AquaVoice Alternative.app`
- `src-tauri/target/release/bundle/dmg/AquaVoice Alternative_0.1.0_aarch64.dmg`

## ライセンス

MIT License

## 貢献

プルリクエストを歓迎します。大きな変更の場合は、まずissueを開いて変更内容を議論してください。
