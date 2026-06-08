# 14. インターコムおよびアクセス制御

## スコープ

インターコムとアクセス制御は重要ですが、MVP では大きく取り込むべきではありません。

v0.1 に含めるもの:

- PIN チェックインイベントモデル
- 接点リレー出力
- スタッフ呼び出しイベントのモック

v0.3 に含める可能性があるもの:

- インターコム API
- SIP 通話モック
- DTMF リレートリガー
- 電話プロバイダー障害シミュレーション
- Zoom Phone モック
- ローカル PBX フォールバック
- ONVIF 風カメライベント
- PoE スピーカー webhook モック

## チェックインモデル

```yaml
checkin:
  guest_id: guest_001
  pin_code: "123456"
  valid_from: 2026-01-01T15:00:00+09:00
  valid_until: 2026-01-02T11:00:00+09:00
```

## 安全なモックフロー

実行可能なスコープはシナリオ専用の安全なモックです。PIN 判定、リレーパルス要求、スタッフ呼び出し試行、フォールバックをエビデンスとして記録しますが、実際の解錠認可や物理ロックの制御は行いません。

```yaml
steps:
  - at: T
    intercom:
      id: front_gate
      event: pin_check
      outcome: accepted
  - at: T+1s
    intercom:
      id: front_gate
      event: relay_pulse
      outcome: requested
  - at: T+5s
    intercom:
      id: front_gate
      event: staff_call
      outcome: failed
      fallback: manual_staff_callback
assertions:
  - at: T+6s
    assert:
      intercom_relay: safe_evidence
```

## 接点リレー出力

```yaml
contacts:
  outputs:
    - id: gate_relay
      type: pulse
      duration_ms: 1000
```

## スタッフ呼び出しモデル

```yaml
staff_call:
  source: intercom.main_gate
  route: online_concierge
  fallback: local_pbx
```

## DTMF（将来）

DTMF はリレーをトリガーする入力イベントとしてモデル化できます。

```yaml
steps:
  - at: T
    call:
      id: staff_call_001
      dtmf: "1"
  - at: T+1s
    assert:
      contact:
        id: gate_relay
        pulsed: true
```

## 電話プロバイダー障害シナリオ（将来）

```yaml
faults:
  - at: T
    target: sip_trunk_provider
    type: offline

assertions:
  - at: T+5s
    target: local_pbx
    condition: used_as_fallback
```

## アクセス制御ドリフトシナリオ

軽量モジュールは、ID プロバイダーのグループメンバーシップとアクセスシステムのユーザーを比較することで、アクセス制御ドリフトをエミュレートできます。

```yaml
scenario:
  name: access_permission_drift

inputs:
  identity_group:
    - alice@example.com
    - bob@example.com
  access_system_group:
    - alice@example.com
    - bob@example.com
    - retired@example.com

assertions:
  - at: T
    assert:
      access_control_drift: detected
```

レポート:

```txt
PASS access_permission_drift

Unexpected access user:
- retired@example.com

Risk:
Former member may still unlock protected space.
```
