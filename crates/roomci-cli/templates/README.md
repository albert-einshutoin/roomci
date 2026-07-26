# roomci scenarios

Start with the runnable MQTT retained-state smoke scenario:

```bash
roomci validate roomci/smoke.yaml
roomci run roomci/smoke.yaml --verbose
```

Write reports for CI or review with:

```bash
roomci run roomci/smoke.yaml --report-dir roomci-reports
```

See https://github.com/albert-einshutoin/roomci/tree/main/docs for scenario,
adapter, and integration guidance.
