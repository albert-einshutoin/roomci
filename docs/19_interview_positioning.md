# 19. Interview Positioning

## One-minute pitch

I studied NOT A HOTEL's smart-home hiring materials, videos, and engineering articles. What stood out is that the smart home is not just device control; it is a local-first, MQTT-driven, edge-server-based, building-automation and operations system that protects the stay experience.

The hardest part is not only writing code. It is making field commissioning, device integration, BMS alerting, network failover, and guest-facing behavior reproducible as the number of sites grows.

So I designed `roomci`: a Docker-based emulator that starts a one-house smart-home stack locally or in CI. It models a local MQTT broker, edge server, iPad controller, Modbus devices, DALI lighting, contact alarms, BMS notifications, network failures, and cloud outage scenarios.

The goal is not to replace production systems. The goal is to make field failures reproducible before guests experience them.

## Japanese pitch

NOT A HOTELのスマートホーム記事・採用情報・動画を見ると、単なるIoTデバイス制御ではなく、Local MQTT、Edge server、制御盤、Modbus、DALI、接点、BMS、Slack/電話通知、ネットワーク冗長化、空調自動化まで含めて、滞在体験を壊さないための総合システムを自社で作っていると理解しました。

特に、拠点数が増える中で、現地QAや施工・保守を人手だけでスケールさせるのではなく、仕組みとして再現可能にする必要があると感じました。

そこで、Docker上で1棟分のスマートホーム環境を立ち上げ、Local MQTT、Edge server、Modbus機器、DALI照明、接点警報、BMS通知、ネットワーク障害をシナリオとして再現し、CIで検証できる `roomci` を設計しました。

## What to emphasize

### 1. You understand the real problem

Not:

```txt
I made an IoT mock server.
```

Better:

```txt
I made a way to turn field QA and smart-home failure modes into reproducible CI scenarios.
```

### 2. You respect the existing stack

Say:

```txt
I am not proposing to replace Go, Lua, Swift, TypeScript, or the production edge stack with Rust. Rust is used as an external deterministic QA engine and Docker image.
```

### 3. You understand local-first reliability

Say:

```txt
The key scenario is cloud outage. If the local iPad, local MQTT broker, and edge server continue working, guest experience can remain intact.
```

### 4. You understand field operations

Mention:

- commissioning
- QA
- runbooks
- alert response
- Slack/phone escalation
- LCM/operations teams
- control panel failure modes
- network failover

### 5. You understand protocol pragmatism

Say:

```txt
I intentionally did not try to implement full KNX/DALI/BACnet compatibility. The MVP models the behavior needed for QA: register maps, scene consistency, contact alerts, and failure reports.
```

## Best demos for interview

1. **Cloud outage but local control works**  
   Shows architecture understanding.

2. **Modbus floor heating register map**  
   Shows field protocol awareness.

3. **DALI scene partial failure**  
   Shows experience-quality awareness.

4. **Sauna emergency contact alert**  
   Shows operations/BMS awareness.

5. **Edge server failover**  
   Shows reliability engineering.

## Avoid saying

- “I reproduced NOT A HOTEL's internal system.”
- “This is fully compatible with KNX/DALI/BACnet.”
- “This replaces real commissioning.”
- “This is a production BMS.”
- “NOT A HOTEL definitely uses X internally” unless confirmed.

## Strong README sentence

```txt
The goal is not to replace real building automation protocols. The goal is to reproduce field failures before guests experience them.
```
