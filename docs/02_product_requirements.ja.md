# 02. 製品要件

## 製品名

`roomci`

## 製品カテゴリ

ローカル開発および CI 向けの Docker ベースのスマートホーム QA・運用エミュレータ。

## 対象ユーザー

- スマートホームソフトウェアエンジニア
- IoT エンジニア
- フィールドエンジニア
- ネットワークエンジニア
- コミッショニングエンジニア
- BMS / LCM / 運用ツールエンジニア
- QA エンジニア
- アプリ、エッジ、デバイス、運用を横断して働くプロダクトエンジニア

## 問題ステートメント

スマートホームシステムはテストが難しい。実際の挙動は次の要素にまたがるからだ:

- モバイルまたはタブレットコントローラ
- ローカルエッジサーバー
- クラウドサービス
- ローカルおよびクラウド MQTT ブローカー
- 産業用またはビルディングオートメーションプロトコル
- 物理デバイス
- 電気接点
- ネットワークインフラ
- BMS アラートと人的運用
- 現場コミッショニングとメンテナンス

多くの障害は現場でのみ発生する。再現にはデバイス、配線、ネットワークトポロジ、運用コンテキストが必要になる。

## 製品目的

スマートホームの現場障害を Docker と CI で再現可能にする。

## 成功基準

### 技術的成功

- 開発者が `docker compose up` を実行し、1 軒分のスマートホームスタックをシミュレートできる。
- シナリオを YAML で表現できる。
- 実デバイスなしでローカル MQTT とエッジサーバーの挙動をテストできる。
- クラウド障害と WAN 障害をシミュレートできる。
- Modbus レジスタマップを検証できる。
- DALI ライクなシーン障害をテストできる。
- 接点 I/O アラートが BMS ライクな通知をトリガーできる。
- レポートを GitHub Actions や CI システムで利用できる。

### 製品成功

- 本ツールが local-first スマートホームアーキテクチャの深い理解を示す。
- コミッショニング知識を実行可能にする。
- プロトコル呼び出しだけでなく、ゲスト体験に影響する障害をモデル化する。
- 想定される現場原因と確認手順を含む有用な障害レポートを提供する。

## MVP 要件

### 機能要件

| ID | 要件 | 優先度 |
|---|---|---|
| FR-001 | シナリオ YAML ファイルを実行する | Must |
| FR-002 | Docker イメージを提供する | Must |
| FR-003 | Docker Compose のサンプルを提供する | Must |
| FR-004 | ローカル MQTT の retained 状態をエミュレートする | Must |
| FR-005 | エッジサーバーのコマンドルーティングをエミュレートする | Must |
| FR-006 | ローカル運用を継続しながらクラウド障害をシミュレートする | Must |
| FR-007 | Modbus TCP レジスタマップをエミュレートする | Must |
| FR-008 | DALI ライクな照明シーンをエミュレートする | Must |
| FR-009 | 接点 I/O アラートをエミュレートする | Must |
| FR-010 | Markdown レポートを生成する | Must |
| FR-011 | JSON レポートを生成する | Must |
| FR-012 | JUnit レポートを生成する | Should |
| FR-013 | QoS1 の重複配信をシミュレートする | Should |
| FR-014 | エッジサーバーのフェイルオーバーをシミュレートする | Should |
| FR-015 | BMS Slack/電話/チケットのモック | Should |
| FR-016 | 快適性自動化モジュール | Could |
| FR-017 | インターコム / インターコム API / SIP モック | Later |

### 非機能要件

| ID | 要件 | 優先度 |
|---|---|---|
| NFR-001 | CI 向けの起動時間 | Must |
| NFR-002 | 決定論的なシナリオ実行 | Must |
| NFR-003 | 単一バイナリまたは軽量コンテナ | Should |
| NFR-004 | 人間が読める障害レポート | Must |
| NFR-005 | 実デバイスへのアクセス不要 | Must |
| NFR-006 | プロトコルモックは明示的な近似であること | Must |
| NFR-007 | 拡張可能なモジュールアーキテクチャ | Should |

## 非目的

- 本番デバイス制御
- KNX / DALI / BACnet プロトコルの完全認証
- 実 SIP トランキング
- デフォルトモードでの実電話プロバイダー / Slack / PagerDuty 呼び出し
- 本番 BMS の置き換え
- 実コミッショニングの置き換え

## MVP リリース定義

次のコマンドが動作すれば v0.1 リリースとして許容する:

```bash
roomci validate examples/local_first_cloud_outage.yaml
roomci run examples/local_first_cloud_outage.yaml --report-md report.md --report-json report.json --junit report.xml

docker compose -f compose/docker-compose.yml up --abort-on-container-exit
```

## UX 原則

レポートはソフトウェアエンジニアとフィールドエンジニアの両方向けに書く。

悪いレポートの例:

```txt
assertion failed: device.state != expected
```

良いレポートの例:

```txt
DALI fixture D411S10 did not reach the expected dimming level.
Possible field causes:
- wrong DALI address
- fixture not assigned to scene group
- wiring issue
- gateway command timeout
```
