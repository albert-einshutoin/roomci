# 評価者向けインテークキット

実顧客 PoC に `roomci` を適合させる前に、このチェックリストを使用します。
顧客が所有するプロトコル情報を、レビュー可能な adapter contract、scenario、
証跡計画へ変換します。非公開製品やベンダーへの対応を意味するものではありません。

本番資格情報、実秘密鍵、制限のないデバイスアクセス、本番専用 endpoint は
要求しません。マスキング済みサンプルと専用の非本番値を使用してください。

## インテーク判定

| 状態 | roomci でできること |
|---|---|
| 顧客仕様なし | 公開の汎用 MQTT、hospitality、building automation、BMS 例を実行し、report 形式と fail-closed 検証を示す |
| 仕様の一部あり | 提供された field だけを検証・モデル化し、不足情報を assumption または blocked acceptance criterion として記録する |
| 完全な非本番仕様あり | 顧客所有の adapter/scenario PoC を作成し、合意した受入基準に対応する証跡を生成する |

顧客所有の仕様がなければ、topic、payload、register、alert、認証、TLS、
timing、安全性、ベンダー互換性を主張できません。公開 fixture の成功が証明するのは
文書化された roomci subset であり、非公開環境との互換性ではありません。

## 必須インテーク

protocol surface ごとに1行ずつ確認します。入手できない入力は `blocked` とし、
非公開値を推測しません。

| 必須入力 | 顧客所有の最小証跡 | Adapter contract の対応先 | Scenario / PoC の対応先 |
|---|---|---|---|
| MQTT topic | マスキング済み command/state topic 例、wildcard/placeholder、方向、QoS、retained/session 前提 | `mqtt.contracts[].name`, `command_topic`, `state_topic`, `adapter` | `mqtt.contracts[]`, `steps[].mqtt_publish`, `assertions[].mqtt`; `make poc-generic-mqtt` |
| MQTT payload | command ごとの accepted/rejected JSON 例、required/optional field、型、enum/range | `mqtt.contracts[].payload.required_fields`, `optional_fields`, `fields` | scenario step または `roomci serve` に publish し、pass/fail report を収集 |
| device identity | device id の取得元、正規化、大文字小文字、許容例、未知 id の挙動 | `devices[].id`, `mqtt.contracts[].device_id_from_topic`, topic の `{device_id}` | 一致/未知 id を scenario 化し、`roomci adapter validate CONTRACT` で検証 |
| Modbus/register map | unit id、address、register type、表現、scale/unit、read/write、範囲、commissioning 値 | `modbus.devices[].id`, `unit_id`, `registers[].address`, `name`, `type`, `access`, `scale`, `unit` | `modbus.devices[]`, `steps[].modbus_write`, Modbus assertion; `make poc-building-automation` |
| BMS alert schema | source id、schema/content type、severity、署名 header、replay window、channel、ticket state、accepted/rejected 例 | `bms.alerts[].id`, `source`, `schema_version`, `content_type`, `severity`, `severity_enum`, `hmac`, `replay_window_seconds`, `channels`, `ticket_lifecycle` | `alerts`, `contacts`, BMS/contact step と assertion; `make poc-bms-ops` |
| 認証/TLS 前提 | test auth mode、secret reference 名、trust source、client auth、hostname/SNI、TLS version、localhost plaintext 可否 | `auth` に `mode`, `secret_ref`, TLS note など非本番前提を記録 | 選択した serve subset が強制しないものは review note とする。現行 HTTP/MQTT serve は localhost 指向で TLS 相互運用性を証明しない |
| 安全境界 | 許可する read/write、禁止操作、非本番 target、rate/concurrency、emergency stop/rollback owner | `edge.commands[]` で modeled command、`devices[]` と Modbus `access` で modeled target を限定 | 承認済み mock/assertion のみを記述。本番 actuator や安全上重要な unlock/control に接続しない |
| report 要件 | 形式、読者、redaction、保持場所、run id、failure diagnostics | `acceptance.report_formats`; 任意の `acceptance.mappings[].artifacts` | CLI report flag または PoC target を選択し、JSON/Markdown/JUnit/timeline/observability/GitHub summary を確認 |
| 受入基準 | 正確な pass/fail 文、stable id、scenario assertion、artifact、timing/tolerance、owner | `acceptance.criteria`; 任意の `acceptance.mappings[].id`, `criterion`, `assertions`, `artifacts` | 参照 assertion に stable `name` を付け、`roomci adapter validate CONTRACT --scenario SCENARIO` で照合 |

## プロトコル詳細チェック

### MQTT

- [ ] command/state topic 例はマスキング済みだが構造は正確である。
- [ ] `{device_id}` の位置と取得規則が明示されている。
- [ ] required、optional、unknown、wrong-type、out-of-range、malformed
  payload の期待挙動が合意されている。
- [ ] QoS、retained state、reconnect、duplicate delivery、session 前提を
  roomci の対応 subset と分けて記載している。

### Modbus

- [ ] transport と unit id が明示されている。
- [ ] 使用する全 address に type、access、scale、unit、byte/word 解釈がある。
- [ ] read-only / writable 境界が承認されている。
- [ ] 未対応 function code と範囲外値の失敗挙動が定義されている。

### BMS / operations

- [ ] alert source と contact の対応が明示されている。
- [ ] schema version、content type、severity enum、required field、
  accepted/rejected sample が提供されている。
- [ ] 署名 header/algorithm と replay window は非本番 secret reference のみを使う。
- [ ] notification channel、ticket lifecycle、acknowledgment、escalation が定義されている。

### 認証、TLS、安全性

- [ ] 認証前提に実資格情報を含めていない。
- [ ] 現行 roomci serve subset が強制できない TLS/trust/client-auth 要件も記録している。
- [ ] 許可/禁止操作を顧客 owner が承認している。
- [ ] PoC を本番と安全上重要な actuator から隔離している。

## PoC contract の作成

1. 公開 template をコピーします。

   ```bash
   cp adapter-contracts/templates/company_adapter_contract.yaml path/to/customer-poc.yaml
   ```

2. 提供された非本番証跡で裏付けられる値だけを置き換えます。
3. adapter contract を検証します。

   ```bash
   cargo run -p roomci-cli -- adapter validate path/to/customer-poc.yaml
   ```

4. stable な acceptance-to-evidence mapping と named assertion を追加し、照合します。

   ```bash
   cargo run -p roomci-cli -- adapter validate \
     path/to/customer-poc.yaml \
     --scenario path/to/customer-scenario.yaml
   ```

5. 顧客所有 scenario の前に、関連する公開 baseline を実行します。

   ```bash
   make poc-generic-mqtt
   make poc-building-automation
   make poc-bms-ops
   ```

6. 顧客所有 scenario を実行し、合意した artifact だけを収集します。

   ```bash
   cargo run -p roomci-cli -- run path/to/customer-scenario.yaml \
     --report-json reports/customer-poc.json \
     --report-md reports/customer-poc.md \
     --junit reports/customer-poc.xml
   ```

## 完了条件

次を満たした場合だけインテーク完了とします。

- 使用する全ての非公開情報に顧客所有の根拠があるか、`blocked` になっている。
- 本番 secret / endpoint を commit していない。
- 各 acceptance criterion に owner と決定的な pass/fail 文がある。
- 全ての主張が named assertion または宣言済み artifact に対応している。
- 未対応 TLS、protocol、timing、安全挙動を推測せず明示している。
- 最終 report が、証跡は提供された非本番仕様だけに適用されると明記している。

field 構文は [Adapter Contract Kit](ADAPTER_CONTRACT_KIT.ja.md)、評価手順は
[Integration Onboarding](INTEGRATION_ONBOARDING.ja.md)、実装 subset と non-goal は
[Protocol Support Matrix](PROTOCOL_SUPPORT_MATRIX.md) を参照してください。
