# Anima Flow — Design Documents

**Your second cortex, always on.**

## 認知ループ

Anima Flow の全体を貫く根本モデル: **観測（Observe）→ 内在化（Internalize）→ 干渉（Intervene）**

- **観測**: Connectors が外部世界を知覚し、情報を取り込む
- **内在化**: Cortex が観測を構造化・記憶・正規化する（**ここが製品の本質**）
- **干渉**: Gateway 経由でユーザーに FB を与え、現実世界に働きかける

## Documents

| ドキュメント | 認知ループ | 内容 |
|-------------|-----------|------|
| [Vision](./vision.md) | — | ビジョン・認知ループモデル・コアバリュー |
| [Architecture](./architecture.md) | 全体 | 認知ループに基づくアーキテクチャ設計 |
| [Context Space](./context-space.md) | 全体 | **文脈空間 — 情報収集と学習のスコープ定義** |
| [Cortex](./cortex.md) | 内在化 | Cortex 設計（Memory / Knowledge Base / SSoT） |
| [Context Engine](./context-engine.md) | 内在化 | **コンテキスト選択エンジン（製品の核心）** |
| [Connectors](./connectors.md) | 観測・干渉 | 外部サービスとの接続（Query / Mutation 分離） |
| [Workflows](./workflows.md) | 全体 | GTD ワークフローと Skills 設計 |
| [Roadmap](./roadmap.md) | — | 実装ロードマップと検討事項 |

## 用語

| 用語 | 意味 | 神経科学での対応 |
|------|------|----------------|
| **認知ループ** | 観測→内在化→干渉の循環プロセス | 知覚→認知→行動ループ |
| **Context Space** | ユーザーが定義する情報収集と学習のスコープ | 注意（attention） |
| **Cortex** | Node と Edge のグラフ全体。内在化レイヤーの中核 | 大脳皮質 |
| **Node** | 情報の最小単位（人物、プロジェクト、エピソード等） | 神経細胞 |
| **Edge** | Node 間の関連。方向と重みを持つ | シナプス |
| **Context Engine** | Cortex から「今何を想起するか」を決めるエンジン | 想起（recall） |
