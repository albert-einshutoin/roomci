.PHONY: demo demo-hospitality demo-generic-mqtt verify docker-demo compose-poc protocol-smoke protocol-smoke-mqtt protocol-smoke-modbus protocol-evidence adapter-samples-smoke poc-generic-mqtt poc-core-qa poc-hospitality poc-building-automation poc-bms-ops clean-reports

HOSPITALITY_SCENARIOS := \
	examples/local_first_cloud_outage.yaml \
	examples/edge_server_failover.yaml \
	examples/modbus_floor_heating.yaml \
	examples/bms_sauna_emergency_alert.yaml \
	examples/starlink_failover.yaml \
	examples/comfort_auto_mode.yaml \
	examples/comfort_timeseries_replay.yaml \
	examples/access_permission_drift.yaml \
	examples/commissioning_checklist.yaml \
	examples/intercom_relay_safe_mock.yaml \
	examples/network_control_panel_fault_profiles.yaml

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
	cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml
	cargo run -p roomci-cli -- run $(PASSING_SCENARIOS)
	@set +e; \
	cargo run -p roomci-cli -- run examples/dali_scene_partial_failure.yaml; \
	status=$$?; \
	set -e; \
	test "$$status" -eq 1
	cargo run -p roomci-cli -- serve --config examples/local_first_cloud_outage.yaml --check
	cargo run -p roomci-cli -- serve --config examples/protocol_conformance_smoke.yaml --check
	docker build -t roomci:verify .
	docker run --rm -v "$$PWD/examples:/scenarios:ro" roomci:verify run \
		/scenarios/local_first_cloud_outage.yaml \
		/scenarios/edge_server_failover.yaml \
		/scenarios/modbus_floor_heating.yaml \
		/scenarios/bms_sauna_emergency_alert.yaml \
		/scenarios/starlink_failover.yaml \
		/scenarios/comfort_auto_mode.yaml \
		/scenarios/comfort_timeseries_replay.yaml \
		/scenarios/access_permission_drift.yaml \
		/scenarios/commissioning_checklist.yaml \
		/scenarios/intercom_relay_safe_mock.yaml \
		/scenarios/network_control_panel_fault_profiles.yaml \
		/scenarios/generic_mqtt_retained_state.yaml \
		/scenarios/generic_mqtt_duplicate_delivery.yaml
	docker compose -f compose/docker-compose.yml build scenario-smoke
	docker compose -f compose/docker-compose.yml run --rm scenario-smoke
	docker compose -f compose/docker-compose.yml build roomci-serve external-controller protocol-smoke
	docker compose -f compose/docker-compose.yml run --rm external-controller
	docker compose -f compose/docker-compose.yml run --rm protocol-smoke
	docker compose -f compose/docker-compose.yml build adapter-samples
	docker compose -f compose/docker-compose.yml run --rm adapter-samples
	docker compose -f compose/docker-compose.yml down
	python3 scripts/protocol_evidence_check.py

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

protocol-smoke: protocol-smoke-mqtt protocol-smoke-modbus

protocol-smoke-mqtt:
	cargo test -p roomci-cli --test cli standard_mqtt_client_publishes_retained_state_through_serve

protocol-smoke-modbus:
	cargo test -p roomci-serve --lib modbus_tcp
	docker compose -f compose/docker-compose.yml build roomci-serve protocol-smoke
	docker compose -f compose/docker-compose.yml run --rm protocol-smoke
	docker compose -f compose/docker-compose.yml down

protocol-evidence:
	python3 scripts/protocol_evidence_check.py

adapter-samples-smoke:
	docker compose -f compose/docker-compose.yml build roomci-serve adapter-samples
	docker compose -f compose/docker-compose.yml run --rm adapter-samples
	docker compose -f compose/docker-compose.yml down

poc-generic-mqtt:
	cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/generic_mqtt_edge_device.yaml
	cargo run -p roomci-cli -- run examples/generic_mqtt_retained_state.yaml --report-json reports/poc_generic_mqtt_retained_state.json --report-md reports/poc_generic_mqtt_retained_state.md --junit reports/poc_generic_mqtt_retained_state.xml
	cargo run -p roomci-cli -- run examples/generic_mqtt_duplicate_delivery.yaml --report-json reports/poc_generic_mqtt_duplicate_delivery.json --report-md reports/poc_generic_mqtt_duplicate_delivery.md --junit reports/poc_generic_mqtt_duplicate_delivery.xml

poc-core-qa:
	cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/hospitality_local_first_room.yaml adapter-contracts/examples/building_automation_bms.yaml
	cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml examples/edge_server_failover.yaml examples/modbus_floor_heating.yaml examples/bms_sauna_emergency_alert.yaml examples/starlink_failover.yaml examples/comfort_auto_mode.yaml examples/comfort_timeseries_replay.yaml examples/access_permission_drift.yaml examples/commissioning_checklist.yaml examples/intercom_relay_safe_mock.yaml examples/network_control_panel_fault_profiles.yaml --report-json reports/poc_core_qa.json --report-md reports/poc_core_qa.md --junit reports/poc_core_qa.xml

poc-hospitality:
	cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/hospitality_local_first_room.yaml
	cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --report-json reports/poc_hospitality_local_first.json --report-md reports/poc_hospitality_local_first.md --junit reports/poc_hospitality_local_first.xml
	cargo run -p roomci-cli -- run examples/edge_server_failover.yaml --report-json reports/poc_hospitality_edge_failover.json --report-md reports/poc_hospitality_edge_failover.md --junit reports/poc_hospitality_edge_failover.xml
	cargo run -p roomci-cli -- run examples/modbus_floor_heating.yaml --report-json reports/poc_hospitality_modbus.json --report-md reports/poc_hospitality_modbus.md --junit reports/poc_hospitality_modbus.xml
	cargo run -p roomci-cli -- run examples/bms_sauna_emergency_alert.yaml --report-json reports/poc_hospitality_bms.json --report-md reports/poc_hospitality_bms.md --junit reports/poc_hospitality_bms.xml
	cargo run -p roomci-cli -- run examples/intercom_relay_safe_mock.yaml --report-json reports/poc_hospitality_intercom_safe_mock.json --report-md reports/poc_hospitality_intercom_safe_mock.md --junit reports/poc_hospitality_intercom_safe_mock.xml
	cargo run -p roomci-cli -- run examples/network_control_panel_fault_profiles.yaml --report-json reports/poc_hospitality_infra_faults.json --report-md reports/poc_hospitality_infra_faults.md --junit reports/poc_hospitality_infra_faults.xml
	cargo run -p roomci-cli -- run examples/comfort_timeseries_replay.yaml --report-json reports/poc_hospitality_comfort_timeseries.json --report-md reports/poc_hospitality_comfort_timeseries.md --junit reports/poc_hospitality_comfort_timeseries.xml
	cargo run -p roomci-cli -- run examples/commissioning_checklist.yaml --report-json reports/poc_hospitality_commissioning.json --report-md reports/poc_hospitality_commissioning.md --junit reports/poc_hospitality_commissioning.xml

poc-building-automation:
	cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/building_automation_bms.yaml
	cargo run -p roomci-cli -- run examples/modbus_floor_heating.yaml --report-json reports/poc_building_modbus.json --report-md reports/poc_building_modbus.md --junit reports/poc_building_modbus.xml
	cargo run -p roomci-cli -- run examples/bms_sauna_emergency_alert.yaml --report-json reports/poc_building_bms_contact.json --report-md reports/poc_building_bms_contact.md --junit reports/poc_building_bms_contact.xml
	cargo run -p roomci-cli -- run examples/network_control_panel_fault_profiles.yaml --report-json reports/poc_building_infra_faults.json --report-md reports/poc_building_infra_faults.md --junit reports/poc_building_infra_faults.xml
	cargo run -p roomci-cli -- run examples/comfort_timeseries_replay.yaml --report-json reports/poc_building_comfort_timeseries.json --report-md reports/poc_building_comfort_timeseries.md --junit reports/poc_building_comfort_timeseries.xml

poc-bms-ops:
	cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/building_automation_bms.yaml adapter-contracts/examples/hospitality_local_first_room.yaml
	cargo run -p roomci-cli -- run examples/bms_sauna_emergency_alert.yaml --report-json reports/poc_bms_ops_alert.json --report-md reports/poc_bms_ops_alert.md --junit reports/poc_bms_ops_alert.xml
	cargo run -p roomci-cli -- run examples/access_permission_drift.yaml --report-json reports/poc_bms_ops_access_drift.json --report-md reports/poc_bms_ops_access_drift.md --junit reports/poc_bms_ops_access_drift.xml

clean-reports:
	rm -f reports/*.json reports/*.md reports/*.xml
