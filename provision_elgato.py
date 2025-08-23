#!/usr/bin/env python3
"""
Elgato Ring Light WiFi Provisioning Script
Configures an Elgato Ring Light to connect to a WiFi network
"""

import json
import requests
import sys
from Crypto.Cipher import AES
from Crypto.Util.Padding import pad
import binascii
import random

DEVICE_IP = "192.168.62.1"
DEVICE_PORT = 9123

def get_device_info():
    """Get device information from Elgato"""
    url = f"http://{DEVICE_IP}:{DEVICE_PORT}/elgato/accessory-info"
    response = requests.get(url)
    return response.json()

def generate_encryption_key(device_info):
    """Generate the encryption key based on device info"""
    base_key = "4CB4btbtB0EADDEEEB2A038A31fwfw56"
    
    firmware_build = device_info['firmwareBuildNumber']
    hardware_type = device_info['hardwareBoardType']
    
    # Convert to hex with proper formatting
    firmware_hex = f"{firmware_build:04x}"
    hardware_hex = f"{hardware_type:04x}"
    
    # Swap bytes (little endian)
    firmware_lsb = firmware_hex[2:] + firmware_hex[:2]
    hardware_lsb = hardware_hex[2:] + hardware_hex[:2]
    
    # Replace placeholders in key
    key = base_key.replace("btbt", hardware_lsb)
    key = key.replace("fwfw", firmware_lsb)
    
    return key

def encrypt_wifi_credentials(ssid, password, security_type, encryption_key):
    """Encrypt WiFi credentials using AES-CBC"""
    # Create JSON payload
    payload = {
        "SSID": ssid,
        "Passphrase": password,
        "SecurityType": str(security_type)
    }
    
    json_str = json.dumps(payload)
    json_bytes = json_str.encode('utf-8')
    
    # Add padding to make it multiple of 16
    padded_data = pad(json_bytes, 16)
    
    # Add random 16-byte prefix
    random_prefix = bytes([random.randint(0, 255) for _ in range(16)])
    data_to_encrypt = random_prefix + padded_data
    
    # Fixed IV used by Elgato
    iv = bytes.fromhex("049F6F1149C6F84B1B14913C71E9CDBE")
    key = bytes.fromhex(encryption_key)
    
    # Encrypt with AES-CBC
    cipher = AES.new(key, AES.MODE_CBC, iv)
    encrypted = cipher.encrypt(data_to_encrypt)
    
    return encrypted

def send_wifi_config(encrypted_data):
    """Send encrypted WiFi configuration to device"""
    url = f"http://{DEVICE_IP}:{DEVICE_PORT}/elgato/wifi-info"
    headers = {'Content-Type': 'application/octet-stream'}
    
    response = requests.put(url, data=encrypted_data, headers=headers)
    return response.status_code == 200

def main():
    print("Elgato Ring Light WiFi Provisioning")
    print("====================================")
    print()
    
    # Get device info
    print("Getting device information...")
    device_info = get_device_info()
    print(f"Device: {device_info['productName']}")
    print(f"Serial: {device_info['serialNumber']}")
    print(f"Firmware: {device_info['firmwareVersion']}")
    print()
    
    # Generate encryption key
    encryption_key = generate_encryption_key(device_info)
    print(f"Generated encryption key: {encryption_key}")
    print()
    
    # Get WiFi credentials from user
    ssid = input("Enter WiFi SSID: ")
    password = input("Enter WiFi Password: ")
    
    # Security type: 0=Open, 1=WEP, 2=WPA/WPA2
    security_type = 2 if password else 0
    
    print()
    print(f"Configuring device to connect to: {ssid}")
    
    # Encrypt credentials
    encrypted_data = encrypt_wifi_credentials(ssid, password, security_type, encryption_key)
    
    # Send to device
    if send_wifi_config(encrypted_data):
        print("✅ WiFi configuration sent successfully!")
        print("The device will now reboot and connect to your network.")
    else:
        print("❌ Failed to send WiFi configuration")
        return 1
    
    return 0

if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)