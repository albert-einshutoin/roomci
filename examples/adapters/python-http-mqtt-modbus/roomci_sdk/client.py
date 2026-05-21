import json
import urllib.error
import urllib.request


class RoomciClient:
    """Small reference client for roomci serve HTTP APIs.

    This is intentionally a reference client for evaluator automation. It does
    not implement production auth, TLS policy, retry budgets, or async IO.
    """

    def __init__(self, base_url="http://127.0.0.1:8080", timeout=5):
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

    def health(self):
        return self._json("GET", "/health")

    def state(self):
        return self._json("GET", "/state")

    def timeline(self):
        return self._json("GET", "/timeline")

    def finish(self):
        return self._json("POST", "/finish")

    def post_bms_contact(
        self,
        source,
        state,
        severity="critical",
        replay_id=None,
    ):
        payload = {
            "source": source,
            "state": state,
            "severity": severity,
        }
        if replay_id is not None:
            payload["replay_id"] = replay_id
        return self._json("POST", "/external/bms/contact", payload)

    def latest_report_json(self):
        return self._json("GET", "/reports/latest.json")

    def _json(self, method, path, payload=None):
        body = None
        headers = {}
        if payload is not None:
            body = json.dumps(payload).encode("utf-8")
            headers["content-type"] = "application/json"
        request = urllib.request.Request(
            f"{self.base_url}{path}",
            data=body,
            headers=headers,
            method=method,
        )
        try:
            with urllib.request.urlopen(request, timeout=self.timeout) as response:
                return json.loads(response.read().decode("utf-8"))
        except urllib.error.HTTPError as error:
            details = error.read().decode("utf-8", errors="replace")
            raise RuntimeError(f"{method} {path} failed: {error.code} {details}") from error
