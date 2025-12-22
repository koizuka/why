# why

コマンドがどのパッケージマネージャーでインストールされたかを特定するCLIツール。

## プロジェクト構造

```
src/
├── main.rs          # CLIエントリーポイント
├── lib.rs           # ライブラリルート
├── cli.rs           # clap引数定義
├── error.rs         # エラー型定義
├── detector/        # コマンド検出ロジック
│   ├── mod.rs
│   ├── path_resolver.rs    # which crate でパス解決
│   └── symlink_analyzer.rs # シンボリックリンク追跡
├── package_managers/ # 各パッケージマネージャー検出
│   ├── mod.rs       # PackageManagerDetector trait
│   ├── homebrew.rs  # macOS/Linux
│   ├── npm.rs       # クロスプラットフォーム
│   ├── bun.rs       # クロスプラットフォーム
│   ├── cargo.rs     # クロスプラットフォーム
│   ├── pip.rs       # クロスプラットフォーム (pipx)
│   ├── go.rs        # クロスプラットフォーム (go install)
│   ├── yarn.rs      # クロスプラットフォーム (yarn global)
│   ├── pnpm.rs      # クロスプラットフォーム (pnpm global)
│   ├── apt.rs       # Linux (Debian/Ubuntu)
│   ├── chocolatey.rs # Windows
│   ├── winget.rs    # Windows
│   ├── scoop.rs     # Windows
│   └── system.rs    # OS標準コマンド
└── platform/
    └── mod.rs       # プラットフォーム判定
```

## 開発コマンド

```bash
cargo build           # ビルド
cargo test            # テスト実行
cargo clippy          # Lint
cargo fmt             # フォーマット
cargo run -- <command> # 実行 (例: cargo run -- git)
```

## CI/CD

- GitHub Actions で Ubuntu/macOS/Windows をテスト
- タグプッシュ (`v*`) で自動リリース

## 対応パッケージマネージャー

- Homebrew (macOS, Linux)
- npm -g
- bun -g
- yarn -g
- pnpm -g
- Cargo (クロスプラットフォーム)
- pipx (クロスプラットフォーム)
- go install (クロスプラットフォーム)
- apt (Debian/Ubuntu)
- Chocolatey (Windows)
- Winget (Windows)
- Scoop (Windows)
- OS標準 (System)
