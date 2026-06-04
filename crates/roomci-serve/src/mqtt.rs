use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use roomci_core::TimelineEvent;
use roomci_scenario::{validate_mqtt_contract_publish, MqttContractPublishError};

use crate::{
    acquire_connection_permit, lock_serve_state, sanitize_external_message_value,
    state_retained_messages_view, ServeError, ServeState, MAX_MQTT_PACKET_BYTES,
    MQTT_CLIENT_TIMEOUT_SECS, MQTT_CONNACK_ACCEPTED, MQTT_CONNACK_UNACCEPTABLE_PROTOCOL,
    MQTT_MAX_INFLIGHT_CONNECTIONS, MQTT_PROTOCOL_LEVEL_3_1_1, MQTT_PROTOCOL_NAME,
};

pub(crate) fn serve_mqtt(
    listener: TcpListener,
    address: SocketAddr,
    state: Arc<Mutex<ServeState>>,
) {
    let active_connections = Arc::new(Mutex::new(0_usize));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let Some(permit) =
                    acquire_connection_permit(&active_connections, MQTT_MAX_INFLIGHT_CONNECTIONS)
                else {
                    eprintln!(
                        "mqtt connection limit reached on {address}; dropping new connection"
                    );
                    continue;
                };
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    let _permit = permit;
                    if let Err(error) = handle_mqtt_client(stream, state) {
                        eprintln!("mqtt request error on {address}: {error}");
                    }
                });
            }
            Err(error) => eprintln!("mqtt accept error on {address}: {error}"),
        }
    }
}

pub(crate) fn handle_mqtt_client(
    mut stream: TcpStream,
    state: Arc<Mutex<ServeState>>,
) -> Result<(), ServeError> {
    let timeout = Some(Duration::from_secs(MQTT_CLIENT_TIMEOUT_SECS));
    stream
        .set_read_timeout(timeout)
        .map_err(|error| ServeError::Runtime(error.to_string()))?;
    stream
        .set_write_timeout(timeout)
        .map_err(|error| ServeError::Runtime(error.to_string()))?;

    loop {
        let Some(packet) = read_mqtt_packet(&mut stream)? else {
            return Ok(());
        };
        match packet.packet_type {
            1 => {
                let return_code = if validate_mqtt_connect(&packet.payload) {
                    MQTT_CONNACK_ACCEPTED
                } else {
                    MQTT_CONNACK_UNACCEPTABLE_PROTOCOL
                };
                stream
                    .write_all(&[0x20, 0x02, 0x00, return_code])
                    .map_err(|error| ServeError::Runtime(error.to_string()))?;
                if return_code != MQTT_CONNACK_ACCEPTED {
                    return Ok(());
                }
            }
            3 => {
                let publish = parse_mqtt_publish(&packet.payload)?;
                let mut state = lock_serve_state(&state)?;
                apply_external_mqtt_publish(&mut state, publish);
            }
            8 => {
                let subscribe = parse_mqtt_subscribe(&packet.payload)?;
                let retained_replay = {
                    let state = lock_serve_state(&state)?;
                    mqtt_retained_replay_for_subscribe(&state, &subscribe)
                };
                stream
                    .write_all(&mqtt_suback_packet(
                        subscribe.packet_id,
                        &retained_replay.return_codes,
                    ))
                    .map_err(|error| ServeError::Runtime(error.to_string()))?;
                for publish in retained_replay.publishes {
                    stream
                        .write_all(&publish)
                        .map_err(|error| ServeError::Runtime(error.to_string()))?;
                }
            }
            12 => {
                stream
                    .write_all(&[0xD0, 0x00])
                    .map_err(|error| ServeError::Runtime(error.to_string()))?;
            }
            14 => return Ok(()),
            _ => {
                return Err(ServeError::Runtime(format!(
                    "unsupported MQTT packet type {}",
                    packet.packet_type
                )));
            }
        }
    }
}

fn validate_mqtt_connect(payload: &[u8]) -> bool {
    if payload.len() < 2 {
        return false;
    }
    let protocol_name_len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    let protocol_name_end = 2 + protocol_name_len;
    if payload.len() <= protocol_name_end {
        return false;
    }
    let Ok(protocol_name) = std::str::from_utf8(&payload[2..protocol_name_end]) else {
        return false;
    };
    let protocol_level = payload[protocol_name_end];
    protocol_name == MQTT_PROTOCOL_NAME && protocol_level == MQTT_PROTOCOL_LEVEL_3_1_1
}

pub(crate) struct MqttPacket {
    pub(crate) packet_type: u8,
    pub(crate) payload: Vec<u8>,
}

pub(crate) struct MqttPublish {
    pub(crate) topic: String,
    pub(crate) payload: BTreeMap<String, serde_json::Value>,
}

pub(crate) struct MqttSubscribe {
    packet_id: u16,
    filters: Vec<String>,
}

pub(crate) struct MqttRetainedReplay {
    pub(crate) return_codes: Vec<u8>,
    pub(crate) publishes: Vec<Vec<u8>>,
}

pub(crate) fn read_mqtt_packet(stream: &mut TcpStream) -> Result<Option<MqttPacket>, ServeError> {
    let mut fixed = [0_u8; 1];
    match stream.read_exact(&mut fixed) {
        Ok(()) => {}
        Err(error) if is_peer_closed(&error) => return Ok(None),
        Err(error) => return Err(ServeError::Runtime(error.to_string())),
    }
    let remaining = read_mqtt_remaining_length(stream)?;
    if remaining > MAX_MQTT_PACKET_BYTES {
        return Err(ServeError::Runtime(format!(
            "MQTT packet too large: {remaining} bytes"
        )));
    }
    let mut payload = vec![0_u8; remaining];
    stream
        .read_exact(&mut payload)
        .map_err(|error| ServeError::Runtime(error.to_string()))?;
    Ok(Some(MqttPacket {
        packet_type: fixed[0] >> 4,
        payload,
    }))
}

fn is_peer_closed(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::ConnectionReset
    )
}

fn read_mqtt_remaining_length(stream: &mut TcpStream) -> Result<usize, ServeError> {
    let mut multiplier = 1_usize;
    let mut value = 0_usize;
    for _ in 0..4 {
        let mut encoded = [0_u8; 1];
        stream
            .read_exact(&mut encoded)
            .map_err(|error| ServeError::Runtime(error.to_string()))?;
        value += ((encoded[0] & 127) as usize) * multiplier;
        if encoded[0] & 128 == 0 {
            return Ok(value);
        }
        multiplier *= 128;
    }
    Err(ServeError::Runtime(
        "malformed MQTT remaining length".to_string(),
    ))
}

pub(crate) fn parse_mqtt_publish(payload: &[u8]) -> Result<MqttPublish, ServeError> {
    if payload.len() < 2 {
        return Err(ServeError::Runtime("malformed MQTT publish".to_string()));
    }
    let topic_len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + topic_len {
        return Err(ServeError::Runtime("malformed MQTT topic".to_string()));
    }
    let topic = String::from_utf8(payload[2..2 + topic_len].to_vec())
        .map_err(|error| ServeError::Runtime(error.to_string()))?;
    let payload_bytes = &payload[2 + topic_len..];
    let payload = serde_json::from_slice::<BTreeMap<String, serde_json::Value>>(payload_bytes)
        .map_err(|error| {
            ServeError::Runtime(format!("MQTT payload must be JSON object: {error}"))
        })?;
    Ok(MqttPublish { topic, payload })
}

pub(crate) fn parse_mqtt_subscribe(payload: &[u8]) -> Result<MqttSubscribe, ServeError> {
    if payload.len() < 5 {
        return Err(ServeError::Runtime("malformed MQTT subscribe".to_string()));
    }
    let packet_id = u16::from_be_bytes([payload[0], payload[1]]);
    if packet_id == 0 {
        return Err(ServeError::Runtime(
            "malformed MQTT subscribe packet id".to_string(),
        ));
    }

    let mut offset = 2;
    let mut filters = Vec::new();
    while offset < payload.len() {
        if payload.len() < offset + 3 {
            return Err(ServeError::Runtime(
                "malformed MQTT subscribe topic".to_string(),
            ));
        }
        let topic_len = u16::from_be_bytes([payload[offset], payload[offset + 1]]) as usize;
        offset += 2;
        if payload.len() < offset + topic_len + 1 {
            return Err(ServeError::Runtime(
                "malformed MQTT subscribe topic".to_string(),
            ));
        }
        let topic_filter = String::from_utf8(payload[offset..offset + topic_len].to_vec())
            .map_err(|error| ServeError::Runtime(error.to_string()))?;
        offset += topic_len;
        let qos = payload[offset];
        offset += 1;
        if qos == 0 {
            filters.push(topic_filter);
        } else {
            filters.push(String::new());
        }
    }

    if filters.is_empty() {
        return Err(ServeError::Runtime("empty MQTT subscribe".to_string()));
    }
    Ok(MqttSubscribe { packet_id, filters })
}

pub(crate) fn apply_external_mqtt_publish(state: &mut ServeState, publish: MqttPublish) {
    state.external_publish_count += 1;
    let event_index = state.external_publish_count;
    let matched = match validate_mqtt_contract_publish(
        &state.scenario.mqtt.contracts,
        &publish.topic,
        &publish.payload,
    ) {
        Ok(matched) => matched,
        Err(error) => {
            let message = match error {
                MqttContractPublishError::TopicMismatch { .. } => {
                    "topic did not match any configured MQTT contract".to_string()
                }
                MqttContractPublishError::MissingRequiredFields { contract, fields } => {
                    format!(
                        "payload missing required fields for {}: {}",
                        sanitize_external_message_value(&contract),
                        fields
                    )
                }
            };
            state.external_observation_timeline.push(TimelineEvent {
                at: format!("external#{event_index}"),
                event_type: "external_mqtt_publish_rejected".to_string(),
                target: Some(sanitize_external_message_value(&publish.topic)),
                message,
            });
            return;
        }
    };

    state
        .external_mqtt_final_state
        .insert(matched.device_id.clone(), publish.payload.clone());
    state
        .external_mqtt_retained_state
        .insert(matched.state_topic.clone(), publish.payload);
    state.external_observation_timeline.push(TimelineEvent {
        at: format!("external#{event_index}"),
        event_type: "external_mqtt_retained_state_updated".to_string(),
        target: Some(sanitize_external_message_value(&matched.device_id)),
        message: format!(
            "external MQTT publish matched {} and updated retained state at {}",
            sanitize_external_message_value(&matched.contract.name),
            sanitize_external_message_value(&matched.state_topic)
        ),
    });
}

pub(crate) fn mqtt_retained_replay_for_subscribe(
    state: &ServeState,
    subscribe: &MqttSubscribe,
) -> MqttRetainedReplay {
    let retained = state_retained_messages_view(state);
    let mut return_codes = Vec::with_capacity(subscribe.filters.len());
    let mut publishes = Vec::new();
    for filter in &subscribe.filters {
        if filter.is_empty() || !is_supported_mqtt_subscription_filter(state, filter) {
            return_codes.push(0x80);
            continue;
        }
        return_codes.push(0x00);
        for (topic, payload) in &retained {
            if mqtt_topic_matches_filter(filter, topic) {
                publishes.push(mqtt_publish_response_packet(topic, payload));
            }
        }
    }
    MqttRetainedReplay {
        return_codes,
        publishes,
    }
}

fn is_supported_mqtt_subscription_filter(state: &ServeState, filter: &str) -> bool {
    state.scenario.mqtt.contracts.iter().any(|contract| {
        filter == contract.state_topic
            || filter == contract.state_topic.replace("{device_id}", "+")
            || state_retained_messages_view(state)
                .keys()
                .any(|topic| mqtt_topic_matches_filter(filter, topic))
    })
}

fn mqtt_topic_matches_filter(filter: &str, topic: &str) -> bool {
    let filter_levels = filter.split('/').collect::<Vec<_>>();
    let topic_levels = topic.split('/').collect::<Vec<_>>();
    if filter_levels.len() != topic_levels.len() {
        return false;
    }
    filter_levels
        .iter()
        .zip(topic_levels.iter())
        .all(|(filter, topic)| *filter == "+" || filter == topic)
}

fn mqtt_suback_packet(packet_id: u16, return_codes: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(2 + return_codes.len());
    payload.extend_from_slice(&packet_id.to_be_bytes());
    payload.extend_from_slice(return_codes);
    let mut packet = vec![0x90];
    encode_mqtt_remaining_length(payload.len(), &mut packet);
    packet.extend(payload);
    packet
}

fn mqtt_publish_response_packet(
    topic: &str,
    payload: &BTreeMap<String, serde_json::Value>,
) -> Vec<u8> {
    let payload = serde_json::to_vec(payload).unwrap_or_else(|_| b"{}".to_vec());
    let mut variable = Vec::with_capacity(2 + topic.len() + payload.len());
    variable.extend_from_slice(&(topic.len() as u16).to_be_bytes());
    variable.extend_from_slice(topic.as_bytes());
    variable.extend_from_slice(&payload);
    let mut packet = vec![0x31];
    encode_mqtt_remaining_length(variable.len(), &mut packet);
    packet.extend(variable);
    packet
}

fn encode_mqtt_remaining_length(mut length: usize, packet: &mut Vec<u8>) {
    loop {
        let mut encoded = (length % 128) as u8;
        length /= 128;
        if length > 0 {
            encoded |= 128;
        }
        packet.push(encoded);
        if length == 0 {
            break;
        }
    }
}
