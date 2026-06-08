# ドメインパック

## コア

再利用可能な `roomci` core はドメイン中立です。

- MQTT/device contracts: command topics、state topics、retained state、duplicate delivery、reconnect 指向の挙動。
- Edge routing/failover: active edge routing、primary/secondary failover、local-first availability。
- Failure injection: cloud outage、local broker outage、duplicate delivery、command drop、edge power loss、WAN failover。
- Report generation: JSON、Markdown、JUnit、timeline、assertions、final state、guest/field impact messages。
- CI execution: deterministic scenario mode、Docker image、Compose smoke tests、quality gates。

ドメインパックは、この core の上に重ねるシナリオ規約です。パックが新しい再利用可能な adapter または assertion type を導入しない限り、runner の変更は不要です。

## ドメインパック

### 汎用 MQTT エッジデバイス

Generic MQTT の例はホスピタリティ向け命名を避け、再利用可能な command/state contract を示します。

例:

- `examples/generic_mqtt_retained_state.yaml`
- `examples/generic_mqtt_duplicate_delivery.yaml`

### ホスピタリティスマートホーム

ホスピタリティシナリオは、ゲスト向け local-first 挙動、客室制御、快適性、現場障害に焦点を当てます。

例:

- `examples/local_first_cloud_outage.yaml`
- `examples/edge_server_failover.yaml`
- `examples/comfort_auto_mode.yaml`
- `examples/dali_scene_partial_failure.yaml`

### ビルディングオートメーション

ビルディングオートメーションシナリオは、完全なプロトコル適合ではなく、実用的な protocol-adapter 挙動をモデル化します。

例:

- `examples/modbus_floor_heating.yaml`
- `examples/dali_scene_partial_failure.yaml`
- `examples/starlink_failover.yaml`

### BMS / 運用

BMS および運用シナリオは、アラート、通知チャネル、チケット状態、runbook を第一級のテスト出力として扱います。

例:

- `examples/bms_sauna_emergency_alert.yaml`
- `examples/starlink_failover.yaml`

### コミッショニング

コミッショニングシナリオは、サイト、客室、デバイス宣言を実行可能なチェックに変換します。

例:

- `examples/commissioning_checklist.yaml`
- `examples/modbus_floor_heating.yaml`

### アクセス制御

アクセス制御シナリオは、実際のアクセスシステムを呼び出さずに identity/access drift を検証します。

例:

- `examples/access_permission_drift.yaml`

## 例マップ

| 例 | ドメインパック | 再利用可能なコントラクト |
|---|---|---|
| `generic_mqtt_retained_state.yaml` | 汎用 MQTT エッジデバイス | コマンド/状態の保持 MQTT コントラクト |
| `generic_mqtt_duplicate_delivery.yaml` | 汎用 MQTT エッジデバイス | 重複配信のべき等性 |
| `local_first_cloud_outage.yaml` | ホスピタリティスマートホーム | クラウド障害時のローカルファースト制御 |
| `edge_server_failover.yaml` | ホスピタリティスマートホーム / ビルディングオートメーション | エッジフェイルオーバーとローカル可用性 |
| `modbus_floor_heating.yaml` | ビルディングオートメーション / コミッショニング | レジスタマップの挙動 |
| `bms_sauna_emergency_alert.yaml` | BMS / 運用 | アラートエスカレーションコントラクト |
| `starlink_failover.yaml` | ビルディングオートメーション / BMS 運用 | WAN フェイルオーバーと運用通知 |
| `comfort_auto_mode.yaml` | ホスピタリティスマートホーム | 快適性自動化コントラクト |
| `access_permission_drift.yaml` | アクセス制御 | ID/アクセスドリフト検知 |
| `commissioning_checklist.yaml` | コミッショニング | 現場チェックリスト生成 |
| `dali_scene_partial_failure.yaml` | ホスピタリティスマートホーム / ビルディングオートメーション | 照明シーン障害検知 |
