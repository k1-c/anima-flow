# Anima Flow — Context Space（文脈空間）

## 概要

Context Space は、ユーザーが定義する**情報収集と学習のスコープ**である。
「何を観測し、何を記憶し、何について考えるか」の境界を明示的に設定する。

Anima Flow の認知ループ（観測→内在化→干渉）は、
Context Space によってスコープされた範囲内で回る。
これにより、無関係な情報のノイズを排除し、
各文脈に集中した高品質な認知を実現する。

## なぜ Context Space が必要か

### Context Space がない世界

```
全コネクタ → 全情報を無差別に取得 → Brain が全件分類 → Cortex に大量保存
```

**問題:**
- Slack の全DM、Linear の全イシュー、全カレンダー予定を毎回取得 → **ノイズだらけ**
- Brain の分類コスト（トークン）が膨大 → **非効率**
- 関連のない情報が Cortex に蓄積 → **想起精度の低下**
- 「今何に集中すべきか」の判断がエージェント任せ → **ユーザーのコントロール喪失**

### Context Space がある世界

```
Context Space で定義されたスコープ → スコープ内の情報のみ取得 → 効率的に内在化
```

**利点:**
- 必要な情報だけを観測 → **高い S/N 比**
- Brain の処理対象が絞られる → **低コスト・高精度**
- Context Space ごとに知識が整理される → **想起精度の向上**
- ユーザーが観測範囲を明示的にコントロール → **信頼と安心**

## Context Space のモデル

### 定義

Context Space は以下の要素で構成される:

```
Context Space
├── name: 文脈空間の名前（例: "プロジェクト Alpha"）
├── description: 目的・概要の説明
├── scopes: コネクタごとの観測範囲
│   ├── slack: { channels: ["#proj-alpha"], keywords: [] }
│   ├── linear: { teams: ["Alpha"], labels: ["alpha"] }
│   ├── todoist: { projects: ["Alpha Tasks"] }
│   ├── calendar: { keywords: ["Alpha", "アルファ"] }
│   └── gmail: { labels: ["alpha"], from: ["@alpha-corp.com"] }
├── status: active | paused | archived
└── created_at / updated_at
```

### Cortex 上の表現

Context Space は Cortex の Node として保存される:

```
Node:
  type: "space"
  category: "ssot"
  title: "プロジェクト Alpha"
  content: "クライアント向け次期プロダクトの開発プロジェクト"
  metadata: {
    "status": "active",
    "scopes": {
      "slack": { "channels": ["#proj-alpha", "#proj-alpha-dev"] },
      "linear": { "teams": ["Alpha"], "labels": ["alpha"] },
      "todoist": { "projects": ["Alpha Tasks"] },
      "calendar": { "keywords": ["Alpha"] }
    }
  }
```

Context Space に関連する Node は Edge で接続される:

```
[Space: プロジェクト Alpha] ←belongs_to→ [Person: 田中さん]
[Space: プロジェクト Alpha] ←belongs_to→ [Episode: Alpha キックオフ MTG]
[Space: プロジェクト Alpha] ←belongs_to→ [Decision: API認証方式の決定]
[Space: プロジェクト Alpha] ←belongs_to→ [Inbox: Linear issue ALP-123]
```

> **Note:** 従来の `project` Node 型は `space` に統合された。
> Context Space は Project Node の上位互換であり、
> 「プロジェクトの現在状態」「意思決定ログ」に加えて
> 「観測スコープ」「ライフサイクル管理」を持つ。

### スコープの型定義

各コネクタのスコープは以下の構造を持つ:

| コネクタ | スコープフィールド | 説明 |
|---------|------------------|------|
| Slack | `channels`, `keywords` | 監視するチャンネルID/名とキーワード |
| Linear | `teams`, `labels`, `projects` | チーム名、ラベル、プロジェクト名 |
| Todoist | `projects`, `labels` | プロジェクト名、ラベル |
| Calendar | `keywords`, `calendar_ids` | イベント名に含むキーワード、カレンダーID |
| Gmail | `labels`, `from`, `subject_keywords` | ラベル、送信元、件名キーワード |
| Chatwork | `room_ids` | 監視するルームID |

## ライフサイクル

### 1. 作成

Context Space の作成は **ユーザー主導** が原則:

```
パターン A: ユーザーが明示的に作成
──────────────────────────────
ユーザー: 「プロジェクト Alpha の Context Space を作って。
           Slack は #proj-alpha、Linear は Alpha チーム」
    ↓
Anima: Context Space "プロジェクト Alpha" を作成
    ↓
観測開始
```

```
パターン B: エージェントが提案 → ユーザーが承認
──────────────────────────────────────────────
ユーザー: 「Slack で新しく来たメッセージの件、プロジェクト化して」
    ↓
Anima: 該当コネクタをスコープなしで一時探索
    ↓
Anima: 「"採用管理" として定義しますか？
        - Slack: #hiring, DM with @tanaka
        - Linear: label "hiring"
        - Calendar: "面接" を含む予定」
    ↓
ユーザー: 承認 / 修正
    ↓
Context Space 作成 → 継続観測開始
```

**重要: エージェントが勝手に Context Space を作成することはない。**
必ずユーザーの明示的な指示または承認を経る。

### 2. 継続的な観測と学習

Context Space が active である間、認知ループが継続的に回る:

```
[Heartbeat / Skill 実行]
    ↓
各 Context Space に対して:
    ↓
Connectors: スコープに基づいて情報収集（Query）
    ↓
Cortex: 新規 Node を作成し、Space に Edge で接続
    ↓
Context Engine: Space 内の Node を優先的に想起
    ↓
Brain: Space の文脈で判断・提案
    ↓
Gateway: ユーザーにフィードバック（干渉）
```

### 3. 一時停止・アーカイブ

```
ユーザー: 「Alpha プロジェクトは一旦止めて」
    ↓
status: active → paused（観測停止、データは保持）

ユーザー: 「Alpha プロジェクトは終了」
    ↓
status: paused → archived（観測停止、検索対象からも除外）
```

### 4. スコープの進化

Context Space のスコープは固定ではなく、使いながら育てていく:

```
Anima: 「#proj-alpha-design でも Alpha 関連の会話が増えています。
        スコープに追加しますか？」
    ↓
ユーザー: 「追加して」
    ↓
スコープ更新: slack.channels に "#proj-alpha-design" を追加
```

この提案も **ユーザーの承認が必要**。

## 認知ループとの関係

### 観測（Connectors）

ConnectorQuery の `fetch_inbox()` は Context Space のスコープを受け取る:

```
fetch_inbox(scope: &ConnectorScope)
    → スコープに該当する情報のみを返す
```

例: Slack Query で `channels: ["#proj-alpha"]` が指定されていれば、
そのチャンネルのメッセージのみを取得する。

### 内在化（Cortex + Context Engine）

- 取得した情報は、対応する Context Space の Node に `belongs_to` Edge で接続される
- Context Engine の想起時、アクティブな Context Space の Node を優先スコアリングする
- Spreading Activation は Space 内の Edge を優先的に辿る

### 干渉（Gateway）

- Skill 実行時、対象の Context Space を明示できる:
  `cargo run -- briefing --space "プロジェクト Alpha"`
- 指定がなければ全 active な Context Space を横断的に処理する

## デフォルト Context Space

初回起動時、以下のデフォルト Context Space が作成される:

```
Context Space: "Personal"
├── description: "個人の日常業務・雑多なタスク"
├── status: active
├── scopes: （全コネクタ、フィルタなし）
```

`Personal` Space は **スコープフィルタなし** で動作する。
ユーザーが明示的な Context Space を定義していない情報は
すべてこの Space に属する。

これにより、Context Space を使わない最小構成でも問題なく動作する。

## Brain との連携

### Context Space を考慮した情報収集

Inbox スキル実行時:

```
1. 全 active Context Space を取得
2. Space ごとに Connectors を呼び出し（スコープ付き）
3. 取得した InboxItem に Space 情報を付与
4. Brain が分類（Space の文脈を踏まえて）
5. Cortex に保存 + Space へ Edge 接続
```

### Context Space の提案

Brain は以下の状況で新しい Context Space を提案できる:

- ユーザーが「〇〇についてプロジェクト化して」と指示した時
- 同じトピックの Inbox アイテムが複数 Space にまたがっている時

提案時、Brain は:
1. 関連するコネクタを一時的にスコープなしで探索
2. 関連情報を集約
3. Space 名、説明、スコープ定義を提案
4. **ユーザーの承認を待つ**

## まとめ

| 概念 | 説明 |
|------|------|
| Context Space | ユーザーが定義する情報収集と学習のスコープ |
| Scope | コネクタごとの観測範囲の設定 |
| Status | active / paused / archived |
| デフォルト | "Personal" Space（フィルタなし） |
| 作成 | ユーザー主導 or エージェント提案→ユーザー承認 |
| 原則 | **エージェントが勝手に作成しない** |
