# Anima Flow

**Your second cortex, always on.**

**観測（Observe）→ 内在化（Internalize）→ 干渉（Intervene）** の認知ループを回し続ける、自律型 AI 秘書エージェント。

## Architecture

```
        ┌──────────────────────────────────────┐
        │            現実世界                    │
        │  Gmail, Slack, Linear, Calendar, ...  │
        └──┬───────────────────────────────┬───┘
           │ 観測                     干渉 │
           ▼                              ▲
    ┌─────────────┐                ┌──────────────┐
    │ Connectors  │                │   Gateway     │
    └──────┬──────┘                └──────▲───────┘
           │                              │
           ▼                              │
    ┌──────────────────────────────────────┴───┐
    │                 Cortex                    │
    │          Node + Edge Graph (Neon)         │
    │     Memory / Knowledge Base / SSoT        │
    │                                           │
    │     Context Engine → Brain → Skills       │
    │     Heartbeat が自律的に駆動              │
    └──────────────────────────────────────────┘
```

## Crates

| Crate | 認知ループ | 役割 |
|-------|-----------|------|
| `anima-core` | — | 共有型・エラー・設定 |
| `anima-cortex` | 内在化 | Node/Edge CRUD + グラフ探索 (Neon Postgres) |
| `anima-context` | 内在化 | Context Engine — 6段パイプラインで想起 |
| `anima-brain` | 駆動 | Anthropic Claude API クライアント |
| `anima-connectors` | 観測 | 外部サービスアダプタ (Gmail, Slack, Linear 等) |
| `anima-gateway` | 干渉 | ユーザーインターフェース (CLI / Slack Bot) |
| `anima-heartbeat` | 駆動 | 自律ループスケジューラ |
| `anima-skills` | 全体 | /morning, /inbox, /breakdown, /review |

## Setup

```bash
# 1. 環境変数
cp .env.example .env
# DATABASE_URL と ANTHROPIC_API_KEY を設定

# 2. マイグレーション
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run

# 3. ビルド & 実行
cargo build
cargo run             # 対話モード
cargo run -- morning  # 朝のブリーフィング
cargo run -- inbox    # Inbox 処理
```

## Design Documents

詳細な設計は [docs/](./docs/index.md) を参照。

- [Vision](./docs/vision.md) — 認知ループモデル・コアバリュー
- [Architecture](./docs/architecture.md) — 認知ループに基づくアーキテクチャ
- [Cortex](./docs/cortex.md) — 内在化レイヤー (Memory / KB / SSoT)
- [Context Engine](./docs/context-engine.md) — 想起エンジン
- [Connectors](./docs/connectors.md) — 観測レイヤー
- [Workflows](./docs/workflows.md) — GTD ワークフロー & Skills
- [Roadmap](./docs/roadmap.md) — 実装ロードマップ

## Tech Stack

- **Rust** (tokio async runtime)
- **Neon (Postgres)** — pgvector, JSONB, FTS, 再帰 CTE
- **Claude API** (Anthropic) — Brain / 推論エンジン
- **sqlx** — コンパイル時 SQL チェック

## License

Private
