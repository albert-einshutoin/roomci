use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use roomci_device_model::{DeviceModelError, ModbusModel};

use crate::{
    acquire_connection_permit, lock_serve_state, ServeError, ServeState,
    MODBUS_CLIENT_TIMEOUT_SECS, MODBUS_EXCEPTION_ILLEGAL_ADDRESS,
    MODBUS_EXCEPTION_ILLEGAL_FUNCTION, MODBUS_EXCEPTION_ILLEGAL_VALUE,
    MODBUS_MAX_INFLIGHT_CONNECTIONS,
};

pub(crate) fn serve_modbus(
    listener: TcpListener,
    address: SocketAddr,
    state: Arc<Mutex<ServeState>>,
) {
    let active_connections = Arc::new(Mutex::new(0_usize));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let Some(permit) =
                    acquire_connection_permit(&active_connections, MODBUS_MAX_INFLIGHT_CONNECTIONS)
                else {
                    eprintln!(
                        "modbus connection limit reached on {address}; dropping new connection"
                    );
                    continue;
                };
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    let _permit = permit;
                    if let Err(error) = handle_modbus_client(stream, state) {
                        eprintln!("modbus request error on {address}: {error}");
                    }
                });
            }
            Err(error) => eprintln!("modbus accept error on {address}: {error}"),
        }
    }
}

fn handle_modbus_client(
    mut stream: TcpStream,
    state: Arc<Mutex<ServeState>>,
) -> Result<(), ServeError> {
    let timeout = Some(Duration::from_secs(MODBUS_CLIENT_TIMEOUT_SECS));
    stream
        .set_read_timeout(timeout)
        .map_err(|error| ServeError::Runtime(error.to_string()))?;
    stream
        .set_write_timeout(timeout)
        .map_err(|error| ServeError::Runtime(error.to_string()))?;

    loop {
        let Some(request) = read_modbus_tcp_request(&mut stream)? else {
            return Ok(());
        };
        let response = {
            let mut state = lock_serve_state(&state)?;
            apply_modbus_tcp_request(&mut state, request)
        };
        stream
            .write_all(&response)
            .map_err(|error| ServeError::Runtime(error.to_string()))?;
    }
}

pub(crate) struct ModbusTcpRequest {
    pub(crate) transaction_id: u16,
    pub(crate) unit_id: u8,
    pub(crate) function: u8,
    pub(crate) payload: Vec<u8>,
}

fn read_modbus_tcp_request(stream: &mut TcpStream) -> Result<Option<ModbusTcpRequest>, ServeError> {
    let mut header = [0_u8; 7];
    match stream.read_exact(&mut header) {
        Ok(()) => {}
        Err(error) if is_peer_closed(&error) => return Ok(None),
        Err(error) => return Err(ServeError::Runtime(error.to_string())),
    }
    let transaction_id = u16::from_be_bytes([header[0], header[1]]);
    let protocol_id = u16::from_be_bytes([header[2], header[3]]);
    if protocol_id != 0 {
        return Err(ServeError::Runtime(
            "invalid Modbus protocol id".to_string(),
        ));
    }
    let length = u16::from_be_bytes([header[4], header[5]]) as usize;
    if length == 0 || length > 260 {
        return Err(ServeError::Runtime("invalid Modbus TCP length".to_string()));
    }
    let unit_id = header[6];
    let mut pdu = vec![0_u8; length - 1];
    stream
        .read_exact(&mut pdu)
        .map_err(|error| ServeError::Runtime(error.to_string()))?;
    let Some((&function, payload)) = pdu.split_first() else {
        return Err(ServeError::Runtime("missing Modbus function".to_string()));
    };
    Ok(Some(ModbusTcpRequest {
        transaction_id,
        unit_id,
        function,
        payload: payload.to_vec(),
    }))
}

fn is_peer_closed(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::ConnectionReset
    )
}

pub(crate) fn apply_modbus_tcp_request(
    state: &mut ServeState,
    request: ModbusTcpRequest,
) -> Vec<u8> {
    match request.function {
        0x03 | 0x04 => apply_modbus_read(state, request),
        0x06 => apply_modbus_write_single_register(state, request),
        function => modbus_exception_response(
            request.transaction_id,
            request.unit_id,
            function,
            MODBUS_EXCEPTION_ILLEGAL_FUNCTION,
        ),
    }
}

fn apply_modbus_read(state: &mut ServeState, request: ModbusTcpRequest) -> Vec<u8> {
    if request.payload.len() != 4 {
        return modbus_exception_response(
            request.transaction_id,
            request.unit_id,
            request.function,
            MODBUS_EXCEPTION_ILLEGAL_VALUE,
        );
    }
    let address = u16::from_be_bytes([request.payload[0], request.payload[1]]) as u32;
    let quantity = u16::from_be_bytes([request.payload[2], request.payload[3]]);
    if quantity == 0 || quantity > 125 {
        return modbus_exception_response(
            request.transaction_id,
            request.unit_id,
            request.function,
            MODBUS_EXCEPTION_ILLEGAL_VALUE,
        );
    }
    let Some(device) = state.modbus_units.get(&request.unit_id).cloned() else {
        return modbus_exception_response(
            request.transaction_id,
            request.unit_id,
            request.function,
            MODBUS_EXCEPTION_ILLEGAL_ADDRESS,
        );
    };
    let first_register =
        normalize_modbus_register(&state.modbus, &device, request.function, address);
    let mut values = Vec::with_capacity(quantity as usize);
    let mut registers = Vec::with_capacity(quantity as usize);
    for offset in 0..quantity {
        let register = first_register + u32::from(offset);
        let Some(value) = state.modbus.value(&device, register).and_then(json_to_u16) else {
            return modbus_exception_response(
                request.transaction_id,
                request.unit_id,
                request.function,
                MODBUS_EXCEPTION_ILLEGAL_ADDRESS,
            );
        };
        values.push(value);
        registers.push(register);
    }
    for register in registers {
        remember_modbus_register(state, &device, register);
    }
    let mut payload = Vec::with_capacity(1 + values.len() * 2);
    payload.push((values.len() * 2) as u8);
    for value in values {
        payload.extend_from_slice(&value.to_be_bytes());
    }
    modbus_success_response(
        request.transaction_id,
        request.unit_id,
        request.function,
        &payload,
    )
}

fn apply_modbus_write_single_register(
    state: &mut ServeState,
    request: ModbusTcpRequest,
) -> Vec<u8> {
    if request.payload.len() != 4 {
        return modbus_exception_response(
            request.transaction_id,
            request.unit_id,
            request.function,
            MODBUS_EXCEPTION_ILLEGAL_VALUE,
        );
    }
    let address = u16::from_be_bytes([request.payload[0], request.payload[1]]) as u32;
    let value = u16::from_be_bytes([request.payload[2], request.payload[3]]);
    let Some(device) = state.modbus_units.get(&request.unit_id).cloned() else {
        return modbus_exception_response(
            request.transaction_id,
            request.unit_id,
            request.function,
            MODBUS_EXCEPTION_ILLEGAL_ADDRESS,
        );
    };
    let register = normalize_modbus_register(&state.modbus, &device, request.function, address);
    let write_value = serde_yaml::to_value(value as i64).unwrap_or(serde_yaml::Value::Null);
    match state.modbus.write(&device, register, write_value) {
        Ok(_) => {
            remember_modbus_register(state, &device, register);
            modbus_success_response(
                request.transaction_id,
                request.unit_id,
                request.function,
                &request.payload,
            )
        }
        Err(DeviceModelError::ReadOnlyModbusRegister { .. }) => modbus_exception_response(
            request.transaction_id,
            request.unit_id,
            request.function,
            MODBUS_EXCEPTION_ILLEGAL_VALUE,
        ),
        Err(
            DeviceModelError::UnknownModbusDevice(_)
            | DeviceModelError::UnknownModbusRegister { .. },
        ) => modbus_exception_response(
            request.transaction_id,
            request.unit_id,
            request.function,
            MODBUS_EXCEPTION_ILLEGAL_ADDRESS,
        ),
        Err(_) => modbus_exception_response(
            request.transaction_id,
            request.unit_id,
            request.function,
            MODBUS_EXCEPTION_ILLEGAL_VALUE,
        ),
    }
}

fn normalize_modbus_register(
    modbus: &ModbusModel,
    device: &str,
    function: u8,
    address: u32,
) -> u32 {
    if modbus.has_register(device, address) {
        return address;
    }
    let conventional = match function {
        0x04 => 30001 + address,
        _ => 40001 + address,
    };
    if modbus.has_register(device, conventional) {
        conventional
    } else {
        address
    }
}

fn remember_modbus_register(state: &mut ServeState, device: &str, register: u32) {
    if let Some(value) = state.modbus.value(device, register).cloned() {
        let mut map = BTreeMap::new();
        map.insert(
            "device".to_string(),
            serde_json::Value::String(device.to_string()),
        );
        map.insert(
            "register".to_string(),
            serde_json::Value::Number(serde_json::Number::from(register)),
        );
        map.insert("value".to_string(), value);
        if let Some(readable) = state.modbus.readable_value(device, register) {
            if let Some(number) = serde_json::Number::from_f64(readable) {
                map.insert(
                    "readable_value".to_string(),
                    serde_json::Value::Number(number),
                );
            }
        }
        state
            .external_modbus_registers
            .insert(format!("modbus.{device}.{register}"), map);
    }
}

fn json_to_u16(value: &serde_json::Value) -> Option<u16> {
    if let Some(value) = value.as_u64() {
        return u16::try_from(value).ok();
    }
    if let Some(value) = value.as_i64() {
        return u16::try_from(value).ok();
    }
    value.as_bool().map(u16::from)
}

fn modbus_success_response(
    transaction_id: u16,
    unit_id: u8,
    function: u8,
    payload: &[u8],
) -> Vec<u8> {
    let mut pdu = Vec::with_capacity(payload.len() + 1);
    pdu.push(function);
    pdu.extend_from_slice(payload);
    modbus_tcp_response(transaction_id, unit_id, &pdu)
}

fn modbus_exception_response(
    transaction_id: u16,
    unit_id: u8,
    function: u8,
    exception: u8,
) -> Vec<u8> {
    modbus_tcp_response(transaction_id, unit_id, &[function | 0x80, exception])
}

pub(crate) fn modbus_tcp_response(transaction_id: u16, unit_id: u8, pdu: &[u8]) -> Vec<u8> {
    let mut response = Vec::with_capacity(7 + pdu.len());
    response.extend_from_slice(&transaction_id.to_be_bytes());
    response.extend_from_slice(&0_u16.to_be_bytes());
    response.extend_from_slice(&((pdu.len() + 1) as u16).to_be_bytes());
    response.push(unit_id);
    response.extend_from_slice(pdu);
    response
}
