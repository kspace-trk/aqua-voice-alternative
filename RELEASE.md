# リリース手順

このドキュメントでは、AquaVoice AlternativeをGitHub Releasesに公開する手順を説明します。

## 自動リリース（推奨）

GitHub Actionsを使用した自動リリースが設定されています。

### 手順

1. バージョン番号を決定（例: `v0.1.0`）

2. タグを作成してプッシュ:
```bash
git tag v0.1.0
git push origin v0.1.0
```

3. GitHub Actionsが自動的に以下を実行:
   - macOS用のビルド（Apple Silicon / Intel）
   - DMGファイルの作成
   - GitHub Releasesへのドラフト作成

4. GitHubのReleasesページでドラフトを確認:
   - https://github.com/kspace-trk/aqua-voice-alternative/releases

5. リリースノートを編集（必要に応じて）

6. 「Publish release」をクリック

## 手動リリース

自動化が利用できない場合の手順です。

### ビルド

```bash
# ローカルでビルド
npm run build

# ビルド成果物の場所
# - src-tauri/target/release/bundle/macos/AquaVoice Alternative.app
# - src-tauri/target/release/bundle/dmg/AquaVoice Alternative_0.1.0_aarch64.dmg
```

### GitHubでリリースを作成

1. https://github.com/kspace-trk/aqua-voice-alternative/releases にアクセス

2. 「Draft a new release」をクリック

3. 以下を入力:
   - **Tag**: 新しいタグを作成（例: `v0.1.0`）
   - **Release title**: バージョン番号（例: `v0.1.0`）
   - **Description**: リリースノート

4. ビルドした`.dmg`ファイルをドラッグ&ドロップ

5. 「Publish release」をクリック

## バージョニング

セマンティックバージョニング（Semantic Versioning）を使用します:

- **メジャーバージョン**: 破壊的な変更
- **マイナーバージョン**: 新機能の追加（後方互換性あり）
- **パッチバージョン**: バグフィックス

例: `v1.2.3`
- `1`: メジャーバージョン
- `2`: マイナーバージョン
- `3`: パッチバージョン

## リリース前のチェックリスト

- [ ] すべてのテストがパス
- [ ] READMEが最新
- [ ] バージョン番号を更新（`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`）
- [ ] CHANGELOGを更新（存在する場合）
- [ ] ローカルでビルドして動作確認
- [ ] コミットをプッシュ

## トラブルシューティング

### ビルドが失敗する

- Rust、Node.js、Tauriのバージョンを確認
- 依存関係を再インストール: `npm install`
- Cargoのキャッシュをクリア: `cargo clean`

### GitHub Actionsが失敗する

- ワークフローのログを確認
- 権限設定を確認（`contents: write`が必要）
- シークレットが設定されているか確認（`GITHUB_TOKEN`は自動的に設定されます）
