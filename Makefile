.PHONY: demo demo-hospitality demo-generic-mqtt verify docker-demo compose-poc clean-reports

HOSPITALITY_SCENARIOS := \
	examples/local_first_cloud_outage.yaml \
	examples/edge_server_failover.yaml \
	examples/modbus_floor_heating.yaml \
	examples/bms_sauna_emergency_alert.yaml \
	examples/starlink_failover.yaml \
	examples/comfort_auto_mode.yaml \
	examples/access_permission_drift.yaml \
	examples/commissioning_checklist.yaml

GENERIC_MQTT_SCENARIOS := \
	examples/generic_mqtt_retained_state.yaml \
	examples/generic_mqtt_duplicate_delivery.yaml

PASSING_SCENARIOS := \
	$(HOSPITALITY_SCENARIOS) \
	$(GENERIC_MQTT_SCENARIOS)

ALL_SCENARIOS := \
	$(PASSING_SCENARIOS) \
	examples/dali_scene_partial_failure.yaml

demo:
	cargo run -p roomci-cli -- run $(PASSING_SCENARIOS)
	@set +e; \
	cargo run -p roomci-cli -- run examples/dali_scene_partial_failure.yaml \
		--report-md reports/dali_scene_partial_failure.md \
		--report-json reports/dali_scene_partial_failure.json \
		--junit reports/dali_scene_partial_failure.xml; \
	status=$$?; \
	set -e; \
	test "$$status" -eq 1

demo-hospitality:
	cargo run -p roomci-cli -- run $(HOSPITALITY_SCENARIOS)

demo-generic-mqtt:
	cargo run -p roomci-cli -- run $(GENERIC_MQTT_SCENARIOS)

verify:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace --all-targets
	RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
	cargo tarpaulin --workspace --engine llvm --fail-under 80
	cargo run -p roomci-cli -- validate $(ALL_SCENARIOS)
	cargo run -p roomci-cli -- run $(PASSING_SCENARIOS)
	@set +e; \
	cargo run -p roomci-cli -- run examples/dali_scene_partial_failure.yaml; \
	status=$$?; \
	set -e; \
	test "$$status" -eq 1
	cargo run -p roomci-cli -- serve --config examples/local_first_cloud_outage.yaml --check
	docker build -t roomci:verify .
	docker run --rm -v "$$PWD/examples:/scenarios:ro" roomci:verify run \
		/scenarios/local_first_cloud_outage.yaml \
		/scenarios/edge_server_failover.yaml \
		/scenarios/modbus_floor_heating.yaml \
		/scenarios/bms_sauna_emergency_alert.yaml \
		/scenarios/starlink_failover.yaml \
		/scenarios/comfort_auto_mode.yaml \
		/scenarios/access_permission_drift.yaml \
		/scenarios/commissioning_checklist.yaml \
		/scenarios/generic_mqtt_retained_state.yaml \
		/scenarios/generic_mqtt_duplicate_delivery.yaml
	docker compose -f compose/docker-compose.yml build scenario-smoke
	docker compose -f compose/docker-compose.yml run --rm scenario-smoke
	docker compose -f compose/docker-compose.yml build roomci-serve external-controller
	docker compose -f compose/docker-compose.yml run --rm external-controller
	docker compose -f compose/docker-compose.yml down

docker-demo:
	docker build -t roomci:demo .
	docker run --rm -v "$$PWD/examples:/scenarios:ro" roomci:demo run \
		/scenarios/local_first_cloud_outage.yaml \
		/scenarios/edge_server_failover.yaml \
		/scenarios/modbus_floor_heating.yaml \
		/scenarios/bms_sauna_emergency_alert.yaml \
		/scenarios/starlink_failover.yaml \
		/scenarios/comfort_auto_mode.yaml \
		/scenarios/access_permission_drift.yaml \
		/scenarios/commissioning_checklist.yaml \
		/scenarios/generic_mqtt_retained_state.yaml \
		/scenarios/generic_mqtt_duplicate_delivery.yaml

compose-poc:
	docker compose -f compose/docker-compose.yml build roomci-serve external-controller
	docker compose -f compose/docker-compose.yml run --rm external-controller
	docker compose -f compose/docker-compose.yml down

clean-reports:
	rm -f reports/*.json reports/*.md reports/*.xml
