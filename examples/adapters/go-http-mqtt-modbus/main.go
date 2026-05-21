package main

import (
	"bytes"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"time"
)

func main() {
	httpBase := env("ROOMCI_HTTP", "http://127.0.0.1:8080")
	mqttAddr := env("ROOMCI_MQTT", "127.0.0.1:1883")
	modbusAddr := env("ROOMCI_MODBUS", "127.0.0.1:1502")

	must(postBMS(httpBase))
	must(publishMQTT(mqttAddr))
	must(readModbus(modbusAddr))
}

func postBMS(base string) error {
	payload := map[string]string{
		"source":         "contact.sauna_emergency_button",
		"state":          "on",
		"severity":       "critical",
		"schema_version": "bms.alert.v1",
		"replay_id":      fmt.Sprintf("go-sample-%d", time.Now().UnixNano()),
	}
	body, err := json.Marshal(payload)
	if err != nil {
		return err
	}
	response, err := http.Post(base+"/external/bms/contact", "application/json", bytes.NewReader(body))
	if err != nil {
		return err
	}
	defer response.Body.Close()
	responseBody, _ := io.ReadAll(response.Body)
	fmt.Printf("BMS POST: %s %s\n", response.Status, bytes.TrimSpace(responseBody))
	if response.StatusCode >= 300 {
		return fmt.Errorf("BMS POST failed with %s", response.Status)
	}
	return nil
}

func publishMQTT(addr string) error {
	conn, err := net.DialTimeout("tcp", addr, 2*time.Second)
	if err != nil {
		return err
	}
	defer conn.Close()
	if _, err := conn.Write(mqttConnectPacket("roomci-go-sample")); err != nil {
		return err
	}
	conn.SetReadDeadline(time.Now().Add(2 * time.Second))
	connack := make([]byte, 4)
	if _, err := io.ReadFull(conn, connack); err != nil {
		return err
	}
	if len(connack) != 4 || connack[0] != 0x20 || connack[3] != 0x00 {
		return fmt.Errorf("unexpected CONNACK: %v", connack)
	}
	topic := "fleet/demo/site/lab/device/env_sensor_01/command"
	payload := []byte(`{"online":true,"sample_interval_seconds":45}`)
	if _, err := conn.Write(mqttPublishPacket(topic, payload)); err != nil {
		return err
	}
	_, _ = conn.Write([]byte{0xe0, 0x00})
	fmt.Println("MQTT PUBLISH: accepted")
	return nil
}

func readModbus(addr string) error {
	conn, err := net.DialTimeout("tcp", addr, 2*time.Second)
	if err != nil {
		return err
	}
	defer conn.Close()
	request := []byte{
		0x00, 0x01, // transaction id
		0x00, 0x00, // protocol id
		0x00, 0x06, // unit id + PDU length
		0x01,       // unit id
		0x03,       // read holding registers
		0x00, 0x00, // start address
		0x00, 0x02, // quantity
	}
	if _, err := conn.Write(request); err != nil {
		return err
	}
	response := make([]byte, 13)
	conn.SetReadDeadline(time.Now().Add(2 * time.Second))
	n, err := conn.Read(response)
	if err != nil {
		return err
	}
	if n < 9 || response[7] != 0x03 {
		return fmt.Errorf("unexpected Modbus response: %v", response[:n])
	}
	registers := make([]uint16, 0, int(response[8])/2)
	for i := 9; i+1 < n; i += 2 {
		registers = append(registers, binary.BigEndian.Uint16(response[i:i+2]))
	}
	fmt.Printf("Modbus read: registers=%v\n", registers)
	return nil
}

func mqttConnectPacket(clientID string) []byte {
	var body bytes.Buffer
	writeMQTTString(&body, "MQTT")
	body.WriteByte(0x04)
	body.WriteByte(0x02)
	body.Write([]byte{0x00, 0x3c})
	writeMQTTString(&body, clientID)
	return append([]byte{0x10, byte(body.Len())}, body.Bytes()...)
}

func mqttPublishPacket(topic string, payload []byte) []byte {
	var body bytes.Buffer
	writeMQTTString(&body, topic)
	body.Write(payload)
	return append([]byte{0x30, byte(body.Len())}, body.Bytes()...)
}

func writeMQTTString(buffer *bytes.Buffer, value string) {
	buffer.WriteByte(byte(len(value) >> 8))
	buffer.WriteByte(byte(len(value)))
	buffer.WriteString(value)
}

func env(key, fallback string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return fallback
}

func must(err error) {
	if err != nil {
		panic(err)
	}
}
