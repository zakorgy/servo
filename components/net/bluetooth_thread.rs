/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use device::bluetooth::BluetoothAdapter;
use device::bluetooth::BluetoothDevice;
use device::bluetooth::BluetoothGATTCharacteristic;
use device::bluetooth::BluetoothGATTDescriptor;
use device::bluetooth::BluetoothGATTService;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use net_traits::bluetooth_scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence, RequestDeviceoptions};
use net_traits::bluetooth_thread::{BluetoothCharacteristicMsg, BluetoothCharacteristicsMsg};
use net_traits::bluetooth_thread::{BluetoothDescriptorMsg, BluetoothDescriptorsMsg};
use net_traits::bluetooth_thread::{BluetoothDeviceMsg, BluetoothError, BluetoothMethodMsg};
use net_traits::bluetooth_thread::{BluetoothResult, BluetoothServiceMsg, BluetoothServicesMsg};
use rand::{self, Rng};
use std::borrow::ToOwned;
use std::collections::{HashMap, HashSet};
use std::string::String;
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
use std::thread;
use std::time::Duration;
#[cfg(target_os = "linux")]
use tinyfiledialogs;
use util::thread::spawn_named;

static TESTING: AtomicBool = ATOMIC_BOOL_INIT;

const ADAPTER_ERROR: &'static str = "No adapter found";

const ADAPTER_NOT_POWERED_ERROR: &'static str = "Bluetooth adapter not powered";

const FAILED_SET_ERROR: &'static str = "Failed to set an attribute for testing";

// A transaction not completed within 30 seconds shall time out. Such a transaction shall be considered to have failed.
// https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=286439 (Vol. 3, page 480)
const MAXIMUM_TRANSACTION_TIME: u8 = 30;
const CONNECTION_TIMEOUT_MS: u64 = 1000;
// The discovery session needs some time to find any nearby devices
const DISCOVERY_TIMEOUT_MS: u64 = 1500;
#[cfg(target_os = "linux")]
const DIALOG_TITLE: &'static str = "Choose a device";
#[cfg(target_os = "linux")]
const DIALOG_COLUMN_ID: &'static str = "Id";
#[cfg(target_os = "linux")]
const DIALOG_COLUMN_NAME: &'static str = "Name";

bitflags! {
    flags Flags: u32 {
        const BROADCAST                   = 0b000000001,
        const READ                        = 0b000000010,
        const WRITE_WITHOUT_RESPONSE      = 0b000000100,
        const WRITE                       = 0b000001000,
        const NOTIFY                      = 0b000010000,
        const INDICATE                    = 0b000100000,
        const AUTHENTICATED_SIGNED_WRITES = 0b001000000,
        const RELIABLE_WRITE              = 0b010000000,
        const WRITABLE_AUXILIARIES        = 0b100000000,
    }
}

macro_rules! return_if_cached(
    ($cache:expr, $key:expr) => (
        if $cache.contains_key($key) {
            return $cache.get($key);
        }
    );
);

macro_rules! get_adapter_or_return_error(
    ($bl_manager:expr, $sender:expr) => (
        match $bl_manager.get_or_create_adapter() {
            Some(adapter) => {
                if !adapter.is_powered().unwrap() {
                    return drop($sender.send(Err(BluetoothError::Type(ADAPTER_NOT_POWERED_ERROR.to_string()))))
                }
                adapter
            },
            None => return drop($sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
        }
    );
);

macro_rules! set_attribute_or_return_error(
    ($function:expr, $sender:expr) => (
        match $function {
            Ok(_) => (),
            Err(_) => return drop($sender.send(Err(BluetoothError::Type(FAILED_SET_ERROR.to_string())))),
        }
    );
);

pub trait BluetoothThreadFactory {
    fn new() -> Self;
}

impl BluetoothThreadFactory for IpcSender<BluetoothMethodMsg> {
    fn new() -> IpcSender<BluetoothMethodMsg> {
        let (sender, receiver) = ipc::channel().unwrap();
        let adapter = BluetoothAdapter::init().ok();
        spawn_named("BluetoothThread".to_owned(), move || {
            BluetoothManager::new(receiver, adapter).start();
        });
        sender
    }
}

fn matches_filter(device: &BluetoothDevice, filter: &BluetoothScanfilter) -> bool {
    if filter.is_empty_or_invalid() {
        return false;
    }

    if !filter.get_name().is_empty() {
        if device.get_name().ok() != Some(filter.get_name().to_string()) {
            return false;
        }
    }

    if !filter.get_name_prefix().is_empty() {
        if let Ok(device_name) = device.get_name() {
            if !device_name.starts_with(filter.get_name_prefix()) {
                return false;
            }
        } else {
            return false;
        }
    }

    if !filter.get_services().is_empty() {
        if let Ok(device_uuids) = device.get_uuids() {
            for service in filter.get_services() {
                if device_uuids.iter().find(|x| x == &service).is_none() {
                    return false;
                }
            }
        }
    }
    return true;
}

fn matches_filters(device: &BluetoothDevice, filters: &BluetoothScanfilterSequence) -> bool {
    if filters.has_empty_or_invalid_filter() {
        return false;
    }

    return filters.iter().any(|f| matches_filter(device, f))
}

pub struct BluetoothManager {
    receiver: IpcReceiver<BluetoothMethodMsg>,
    adapter: Option<BluetoothAdapter>,
    address_to_id: HashMap<String, String>,
    service_to_device: HashMap<String, String>,
    characteristic_to_service: HashMap<String, String>,
    descriptor_to_characteristic: HashMap<String, String>,
    cached_devices: HashMap<String, BluetoothDevice>,
    cached_services: HashMap<String, BluetoothGATTService>,
    cached_characteristics: HashMap<String, BluetoothGATTCharacteristic>,
    cached_descriptors: HashMap<String, BluetoothGATTDescriptor>,
    allowed_services: HashMap<String, HashSet<String>>,
}

impl BluetoothManager {
    pub fn new (receiver: IpcReceiver<BluetoothMethodMsg>, adapter: Option<BluetoothAdapter>) -> BluetoothManager {
        BluetoothManager {
            receiver: receiver,
            adapter: adapter,
            address_to_id: HashMap::new(),
            service_to_device: HashMap::new(),
            characteristic_to_service: HashMap::new(),
            descriptor_to_characteristic: HashMap::new(),
            cached_devices: HashMap::new(),
            cached_services: HashMap::new(),
            cached_characteristics: HashMap::new(),
            cached_descriptors: HashMap::new(),
            allowed_services: HashMap::new(),
        }
    }

    fn start(&mut self) {
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                BluetoothMethodMsg::RequestDevice(options, sender) => {
                    self.request_device(options, sender)
                },
                BluetoothMethodMsg::GATTServerConnect(device_id, sender) => {
                    self.gatt_server_connect(device_id, sender)
                },
                BluetoothMethodMsg::GATTServerDisconnect(device_id, sender) => {
                    self.gatt_server_disconnect(device_id, sender)
                },
                BluetoothMethodMsg::GetPrimaryService(device_id, uuid, sender) => {
                    self.get_primary_service(device_id, uuid, sender)
                },
                BluetoothMethodMsg::GetPrimaryServices(device_id, uuid, sender) => {
                    self.get_primary_services(device_id, uuid, sender)
                },
                BluetoothMethodMsg::GetIncludedService(service_id, uuid, sender) => {
                    self.get_included_service(service_id, uuid, sender)
                },
                BluetoothMethodMsg::GetIncludedServices(service_id, uuid, sender) => {
                    self.get_included_services(service_id, uuid, sender)
                },
                BluetoothMethodMsg::GetCharacteristic(service_id, uuid, sender) => {
                    self.get_characteristic(service_id, uuid, sender)
                },
                BluetoothMethodMsg::GetCharacteristics(service_id, uuid, sender) => {
                    self.get_characteristics(service_id, uuid, sender)
                },
                BluetoothMethodMsg::GetDescriptor(characteristic_id, uuid, sender) => {
                    self.get_descriptor(characteristic_id, uuid, sender)
                },
                BluetoothMethodMsg::GetDescriptors(characteristic_id, uuid, sender) => {
                    self.get_descriptors(characteristic_id, uuid, sender)
                },
                BluetoothMethodMsg::ReadValue(id, sender) => {
                    self.read_value(id, sender)
                },
                BluetoothMethodMsg::WriteValue(id, value, sender) => {
                    self.write_value(id, value, sender)
                },
                BluetoothMethodMsg::Test(data_set_name, sender) => {
                    self.test(data_set_name, sender)
                }
                BluetoothMethodMsg::Exit => {
                    break
                },
            }
        }
    }

    // Test

    fn test(&mut self, data_set_name: String, sender: IpcSender<BluetoothResult<bool>>) {
        TESTING.fetch_or(true, Ordering::Relaxed);
        self.adapter = BluetoothAdapter::init().ok();
        match data_set_name.as_str() {
            "NotPresentAdapter" => {
                match self.adapter.as_ref() {
                    Some(ref adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(String::from("NotPresentAdapter")), sender);
                        set_attribute_or_return_error!(adapter.set_present(false), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                        set_attribute_or_return_error!(adapter.set_powered(true), sender);
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            "NotPoweredAdapter" => {
                match self.adapter.as_ref() {
                    Some(ref adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(String::from("NotPoweredAdapter")), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            "EmptyAdapter" => {
                match self.adapter.as_ref() {
                    Some(ref adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(String::from("EmptyAdapter")), sender);
                        set_attribute_or_return_error!(adapter.set_powered(true), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            "FailStartDiscoveryAdapter" => {
                match self.adapter.as_ref() {
                    Some(ref adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(String::from("FailStartDiscoveryAdapter")),
                                                       sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            //"FailStopDiscoveryAdapter" => {
            //},
            "GlucoseHeartRateAdapter" => {
                let random_id_1 = self.generate_device_id();
                let random_id_2 = self.generate_device_id();
                match self.adapter.as_ref() {
                    Some(adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(String::from("GlucoseHeartRateAdapter"
                                                       .to_owned())),
                                                       sender);
                        set_attribute_or_return_error!(adapter.set_powered(true), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                        let glucose_device = BluetoothDevice::create_device(adapter.clone(), random_id_1);
                        set_attribute_or_return_error!(glucose_device.set_name("Glucose Device".to_owned()), sender);
                        set_attribute_or_return_error!(glucose_device.set_address("00:00:00:00:00:01".to_owned()),
                                                       sender);
                        // Generic Acces, Glucose UUID, Tx Power
                        set_attribute_or_return_error!(glucose_device.set_uuids(
                            vec!("00001800-0000-1000-8000-00805f9b34fb".to_owned(),
                                 "00001808-0000-1000-8000-00805f9b34fb".to_owned(),
                                 "00001804-0000-1000-8000-00805f9b34fb".to_owned())),
                                                      sender);
                        let heart_rate_device = BluetoothDevice::create_device(adapter.clone(), random_id_2);
                        set_attribute_or_return_error!(heart_rate_device.set_name("Heart Rate Device".to_owned()),
                                                       sender);
                        set_attribute_or_return_error!(heart_rate_device.set_address("00:00:00:00:00:02".to_owned()),
                                                       sender);
                        set_attribute_or_return_error!(heart_rate_device.set_connectable(true), sender);
                        // Generic Acces, Heart Rate UUID
                        set_attribute_or_return_error!(heart_rate_device.set_uuids(
                            vec!("00001800-0000-1000-8000-00805f9b34fb".to_owned(),
                                 "0000180d-0000-1000-8000-00805f9b34fb".to_owned())),
                                                       sender);
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },

            "UnicodeDeviceAdapter" => {
                let random_id = self.generate_device_id();
                match self.adapter.as_ref() {
                    Some(adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(String::from("GlucoseHeartRateAdapter"
                                                                        .to_owned())),
                                                       sender);
                        set_attribute_or_return_error!(adapter.set_powered(true), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                        let unicode_device = BluetoothDevice::create_device(adapter.clone(), random_id);
                        set_attribute_or_return_error!(unicode_device.set_name(
                            "❤❤❤❤❤❤❤❤❤".to_owned()),
                                                       sender);
                        set_attribute_or_return_error!(unicode_device.set_address("00:00:00:00:00:03".to_owned()),
                                                       sender);
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            "BlacklistedServicesAdapter" => {
                let random_id = self.generate_device_id();
                match self.adapter.as_ref() {
                    Some(adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(String::from("BlacklistedServicesAdapter"
                                                                                     .to_owned())),
                                                       sender);
                        set_attribute_or_return_error!(adapter.set_powered(true), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                        let device = BluetoothDevice::create_device(adapter.clone(), random_id);
                        set_attribute_or_return_error!(device.set_name("Mock Device".to_owned()), sender);
                        set_attribute_or_return_error!(device.set_address("00:00:00:00:00:04".to_owned()), sender);
                        set_attribute_or_return_error!(device.set_connectable(true), sender);
                        set_attribute_or_return_error!(device.set_uuids(
                            vec!("00001812-0000-1000-8000-00805f9b34fb".to_owned(),
                                 "00001530-1212-efde-1523-785feabcd123".to_owned(),
                                 "f000ffc0-0451-4000-b000-000000000000".to_owned())),
                                                       sender);
                        let _human_interface_device =
                            BluetoothGATTService::new_mock(device.clone(),
                                                           "00001812-0000-1000-8000-00805f9b34fb".to_owned());
                        let _firmware_update_service =
                            BluetoothGATTService::new_mock(device.clone(),
                                                           "00001530-1212-efde-1523-785feabcd123".to_owned());
                        let _over_the_air_download_service =
                            BluetoothGATTService::new_mock(device.clone(),
                                                           "f000ffc0-0451-4000-b000-000000000000".to_owned());
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            "MissingCharacteristicGenericAccessAdapter" => {
                let random_id = self.generate_device_id();
                match self.adapter.as_ref() {
                    Some(adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(
                            String::from("MissingCharacteristicGenericAccessAdapter".to_owned())),
                                                       sender);
                        set_attribute_or_return_error!(adapter.set_powered(true), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                        let heart_rate_device = BluetoothDevice::create_device(adapter.clone(), random_id);
                        set_attribute_or_return_error!(heart_rate_device.set_name("Heart Rate Device".to_owned()),
                                                       sender);
                        set_attribute_or_return_error!(heart_rate_device.set_address("00:00:00:00:00:05".to_owned()),
                                                       sender);
                        set_attribute_or_return_error!(heart_rate_device.set_connectable(true), sender);
                        // Generic Acces, Heart Rate UUID
                        set_attribute_or_return_error!(heart_rate_device.set_uuids(
                            vec!("00001800-0000-1000-8000-00805f9b34fb".to_owned(),
                                 "0000180d-0000-1000-8000-00805f9b34fb".to_owned())),
                                                       sender);
                        let _generic_access_service =
                            BluetoothGATTService::new_mock(heart_rate_device.clone(),
                                                           "00001800-0000-1000-8000-00805f9b34fb".to_owned());
                        let _heart_rate_service =
                            BluetoothGATTService::new_mock(heart_rate_device.clone(),
                                                           "0000180d-0000-1000-8000-00805f9b34fb".to_owned());
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            "MissingDescriptorGenericAccessAdapter" => {
                let random_id = self.generate_device_id();
                match self.adapter.as_ref() {
                    Some(adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(
                            String::from("MissingDescriptorGenericAccessAdapter".to_owned())),
                                                       sender);
                        set_attribute_or_return_error!(adapter.set_powered(true), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                        let heart_rate_device = BluetoothDevice::create_device(adapter.clone(), random_id);
                        set_attribute_or_return_error!(heart_rate_device.set_name("Heart Rate Device".to_owned()),
                                                       sender);
                        set_attribute_or_return_error!(heart_rate_device.set_address("00:00:00:00:00:06".to_owned()),
                                                       sender);
                        set_attribute_or_return_error!(heart_rate_device.set_connectable(true), sender);
                        // Generic Acces, Heart Rate UUID
                        set_attribute_or_return_error!(heart_rate_device.set_uuids(
                            vec!("00001800-0000-1000-8000-00805f9b34fb".to_owned(),
                                 "0000180d-0000-1000-8000-00805f9b34fb".to_owned())),
                                                       sender);
                        let generic_access_service =
                            BluetoothGATTService::new_mock(heart_rate_device.clone(),
                                                           "00001800-0000-1000-8000-00805f9b34fb".to_owned());
                        let heart_rate_service =
                            BluetoothGATTService::new_mock(heart_rate_device.clone(),
                                                           "0000180d-0000-1000-8000-00805f9b34fb".to_owned());

                        let device_name_characteristic =
                            BluetoothGATTCharacteristic::new_mock(generic_access_service.clone(),
                                                                  "00002a00-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(device_name_characteristic.write_value(vec![1]), sender);

                        let pheripheral_privacy_flag_characteristic =
                            BluetoothGATTCharacteristic::new_mock(generic_access_service.clone(),
                                                                  "00002a02-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(pheripheral_privacy_flag_characteristic.write_value(vec![2]),
                                                       sender);

                        let heart_rate_measurement_characteristic =
                            BluetoothGATTCharacteristic::new_mock(heart_rate_service.clone(),
                                                                  "00002a37-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(heart_rate_measurement_characteristic.write_value(vec![3]),
                                                       sender);

                        let body_sensor_location_characteristic_1 =
                            BluetoothGATTCharacteristic::new_mock(heart_rate_service.clone(),
                                                                  "00002a38-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(body_sensor_location_characteristic_1.write_value(vec![4]),
                                                       sender);

                        let body_sensor_location_characteristic_2 =
                            BluetoothGATTCharacteristic::new_mock(heart_rate_service.clone(),
                                                                  "00002a38-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(body_sensor_location_characteristic_2.write_value(vec![5]),
                                                       sender);
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            "ExcludedForWritesCharacteristicAdapter" => {
                let random_id = self.generate_device_id();
                match self.adapter.as_ref() {
                    Some(adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(
                            String::from("ExcludedForWritesCharacteristicAdapter".to_owned())),
                                                       sender);
                        set_attribute_or_return_error!(adapter.set_powered(true), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                        let device = BluetoothDevice::create_device(adapter.clone(), random_id);
                        set_attribute_or_return_error!(device.set_name("Mock Device".to_owned()), sender);
                        set_attribute_or_return_error!(device.set_address("00:00:00:00:00:07".to_owned()), sender);
                        set_attribute_or_return_error!(device.set_connectable(true), sender);
                        // Reconnection Address, Peripheral Privacy Flag
                        set_attribute_or_return_error!(device.set_uuids(
                            vec!("00001800-0000-1000-8000-00805f9b34fb".to_owned())),
                                                       sender);
                        let service =
                            BluetoothGATTService::new_mock(device.clone(),
                                                           "00001800-0000-1000-8000-00805f9b34fb".to_owned());

                        let pheripheral_privacy_flag_characteristic =
                            BluetoothGATTCharacteristic::new_mock(service.clone(),
                                                                  "00002a02-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(pheripheral_privacy_flag_characteristic.write_value(vec![8]),
                                                       sender);
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            "BlacklistedCharacteristicsAdapter" => {
                let random_id = self.generate_device_id();
                match self.adapter.as_ref() {
                    Some(adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(
                            String::from("BlacklistedCharacteristicsAdapter".to_owned())),
                                                       sender);
                        set_attribute_or_return_error!(adapter.set_powered(true), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                        let device = BluetoothDevice::create_device(adapter.clone(), random_id);
                        set_attribute_or_return_error!(device.set_name("Mock Device".to_owned()), sender);
                        set_attribute_or_return_error!(device.set_address("00:00:00:00:00:08".to_owned()), sender);
                        set_attribute_or_return_error!(device.set_connectable(true), sender);
                        // Reconnection Address, Serial Number String
                        set_attribute_or_return_error!(device.set_uuids(
                            vec!("00001800-0000-1000-8000-00805f9b34fb".to_owned())),
                                                       sender);
                        let service =
                            BluetoothGATTService::new_mock(device.clone(),
                                                           "00001800-0000-1000-8000-00805f9b34fb".to_owned());

                        let reconnection_address_characteristic =
                            BluetoothGATTCharacteristic::new_mock(service.clone(),
                                                                  "00002a03-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(reconnection_address_characteristic.write_value(vec![6]),
                                                       sender);

                        let serial_number_string_characteristic =
                            BluetoothGATTCharacteristic::new_mock(service.clone(),
                                                                  "00002a25-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(serial_number_string_characteristic.write_value(vec![7]),
                                                       sender);
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            "CompletedAdapter" => {
                let random_id = self.generate_device_id();
                match self.adapter.as_ref() {
                    Some(adapter) => {
                        set_attribute_or_return_error!(adapter.set_name(String::from("CompletedAdapter".to_owned())),
                                                       sender);
                        set_attribute_or_return_error!(adapter.set_powered(true), sender);
                        set_attribute_or_return_error!(adapter.set_discoverable(true), sender);
                        let heart_rate_device = BluetoothDevice::create_device(adapter.clone(), random_id);
                        set_attribute_or_return_error!(heart_rate_device.set_name("Heart Rate Device".to_owned()),
                                                       sender);
                        set_attribute_or_return_error!(heart_rate_device.set_address("00:00:00:00:00:09".to_owned()),
                                                       sender);
                        set_attribute_or_return_error!(heart_rate_device.set_connectable(true), sender);
                        // Generic Acces, Heart Rate UUID
                        set_attribute_or_return_error!(heart_rate_device.set_uuids(
                            vec!("00001800-0000-1000-8000-00805f9b34fb".to_owned(),
                                 "0000180d-0000-1000-8000-00805f9b34fb".to_owned())),
                                                       sender);
                        let generic_access_service =
                            BluetoothGATTService::new_mock(heart_rate_device.clone(),
                                                           "00001800-0000-1000-8000-00805f9b34fb".to_owned());
                        let heart_rate_service =
                            BluetoothGATTService::new_mock(heart_rate_device.clone(),
                                                           "0000180d-0000-1000-8000-00805f9b34fb".to_owned());

                        let device_name_characteristic =
                            BluetoothGATTCharacteristic::new_mock(generic_access_service.clone(),
                                                                  "00002A00-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(device_name_characteristic.write_value(vec![9]), sender);

                        let pheripheral_privacy_flag_characteristic =
                            BluetoothGATTCharacteristic::new_mock(generic_access_service.clone(),
                                                                  "00002A02-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(pheripheral_privacy_flag_characteristic.write_value(vec![10]),
                                                       sender);

                        let heart_rate_measurement_characteristic =
                            BluetoothGATTCharacteristic::new_mock(heart_rate_service.clone(),
                                                                  "00002a37-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(heart_rate_measurement_characteristic.write_value(vec![11]),
                                                       sender);

                        let body_sensor_location_characteristic_1 =
                            BluetoothGATTCharacteristic::new_mock(heart_rate_service.clone(),
                                                                  "00002a38-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(body_sensor_location_characteristic_1.write_value(vec![12]),
                                                       sender);

                        let body_sensor_location_characteristic_2 =
                            BluetoothGATTCharacteristic::new_mock(heart_rate_service.clone(),
                                                                  "00002a38-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(body_sensor_location_characteristic_2.write_value(vec![13]),
                                                       sender);

                        let desc_for_hrmeasurement_descriptor =
                            BluetoothGATTDescriptor::new_mock(heart_rate_measurement_characteristic.clone(),
                                                              "00002901-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(desc_for_hrmeasurement_descriptor.write_value(vec![14]),
                                                       sender);

                        let desc_for_bslocation_descriptor =
                            BluetoothGATTDescriptor::new_mock(body_sensor_location_characteristic_1.clone(),
                                                              "00002901-0000-1000-8000-00805f9b34fb".to_owned());
                        set_attribute_or_return_error!(desc_for_bslocation_descriptor.write_value(vec![15]),
                                                       sender);
                    },
                    None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
                }
            },
            _ => unreachable!(),
        }
    }

    // Adapter

    fn get_or_create_adapter(&mut self) -> Option<BluetoothAdapter> {
        let adapter_valid = self.adapter.as_ref().map_or(false, |a| a.get_address().is_ok());
        if !adapter_valid {
            self.adapter = BluetoothAdapter::init().ok();
        }

        let adapter = match self.adapter.as_ref() {
            Some(adapter) => adapter,
            None => return None,
        };

        if TESTING.load(Ordering::Relaxed) && !adapter.is_present().unwrap_or(true) {
            return None;
        }

        self.adapter.clone()
    }

    // Device

    fn get_and_cache_devices(&mut self, adapter: &mut BluetoothAdapter) -> Vec<BluetoothDevice> {
        let devices = adapter.get_devices().unwrap_or(vec!());
        for device in &devices {
            if let Ok(address) = device.get_address() {
                if !self.address_to_id.contains_key(&address) {
                    let generated_id = self.generate_device_id();
                    self.address_to_id.insert(address, generated_id.clone());
                    self.cached_devices.insert(generated_id.clone(), device.clone());
                    self.allowed_services.insert(generated_id, HashSet::new());
                }
            }
        }
        self.cached_devices.iter().map(|(_, d)| d.clone()).collect()
    }

    fn get_device(&mut self, adapter: &mut BluetoothAdapter, device_id: &str) -> Option<&BluetoothDevice> {
        return_if_cached!(self.cached_devices, device_id);
        self.get_and_cache_devices(adapter);
        return_if_cached!(self.cached_devices, device_id);
        None
    }

    #[cfg(target_os = "linux")]
    fn select_device(&mut self, devices: Vec<BluetoothDevice>) -> Option<String> {
        if TESTING.load(Ordering::Relaxed) {
            for device in devices {
                if let Ok(address) = device.get_address() {
                    return Some(address);
                }
            }
            return None;
        }
        let mut dialog_rows: Vec<String> = vec!();
        for device in devices {
            dialog_rows.extend_from_slice(&[device.get_address().unwrap_or("".to_string()),
                                            device.get_name().unwrap_or("".to_string())]);
        }
        let dialog_rows: Vec<&str> = dialog_rows.iter()
                                                .map(|s| s.as_ref())
                                                .collect();
        let dialog_rows: &[&str] = dialog_rows.as_slice();

        if let Some(device) = tinyfiledialogs::list_dialog(DIALOG_TITLE,
                                                           &[DIALOG_COLUMN_ID, DIALOG_COLUMN_NAME],
                                                           Some(dialog_rows)) {
            // The device string format will be "Address|Name". We need the first part of it.
            return device.split("|").next().map(|s| s.to_string());
        }
        None
    }

    #[cfg(not(target_os = "linux"))]
    fn select_device(&mut self, devices: Vec<BluetoothDevice>) -> Option<String> {
        for device in devices {
            if let Ok(address) = device.get_address() {
                return Some(address);
            }
        }
        None
    }

    fn generate_device_id(&mut self) -> String {
        let mut device_id;
        let mut rng = rand::thread_rng();
        loop {
            device_id = rng.gen::<u32>().to_string();
            if !self.cached_devices.contains_key(&device_id) {
                break;
            }
        }
        device_id
    }

    // Service

    fn get_and_cache_gatt_services(&mut self,
                                   adapter: &mut BluetoothAdapter,
                                   device_id: &str)
                                   -> Vec<BluetoothGATTService> {
        let services = match self.get_device(adapter, device_id) {
            Some(d) => d.get_gatt_services().unwrap_or(vec!()),
            None => vec!(),
        };
        for service in &services {
            self.cached_services.insert(service.get_id(), service.clone());
            self.service_to_device.insert(service.get_id(), device_id.to_owned());
        }
        services
    }

    fn get_gatt_service(&mut self, adapter: &mut BluetoothAdapter, service_id: &str) -> Option<&BluetoothGATTService> {
        return_if_cached!(self.cached_services, service_id);
        let device_id = match self.service_to_device.get(service_id) {
            Some(d) => d.clone(),
            None => return None,
        };
        self.get_and_cache_gatt_services(adapter, &device_id);
        return_if_cached!(self.cached_services, service_id);
        None
    }

    fn get_gatt_services_by_uuid(&mut self,
                                 adapter: &mut BluetoothAdapter,
                                 device_id: &str,
                                 service_uuid: &str)
                                 -> Vec<BluetoothGATTService> {
        let services = self.get_and_cache_gatt_services(adapter, device_id);
        services.into_iter().filter(|s| s.get_uuid().ok() == Some(service_uuid.to_string())).collect()
    }

    // Characteristic

    fn get_and_cache_gatt_characteristics(&mut self,
                                          adapter: &mut BluetoothAdapter,
                                          service_id: &str)
                                          -> Vec<BluetoothGATTCharacteristic> {
        let characteristics = match self.get_gatt_service(adapter, service_id) {
            Some(s) => s.get_gatt_characteristics().unwrap_or(vec!()),
            None => vec!(),
        };

        for characteristic in &characteristics {
            self.cached_characteristics.insert(characteristic.get_id(), characteristic.clone());
            self.characteristic_to_service.insert(characteristic.get_id(), service_id.to_owned());
        }
        characteristics
    }

    fn get_gatt_characteristic(&mut self,
                               adapter: &mut BluetoothAdapter,
                               characteristic_id: &str)
                               -> Option<&BluetoothGATTCharacteristic> {
        return_if_cached!(self.cached_characteristics, characteristic_id);
        let service_id = match self.characteristic_to_service.get(characteristic_id) {
            Some(s) => s.clone(),
            None => return None,
        };
        self.get_and_cache_gatt_characteristics(adapter, &service_id);
        return_if_cached!(self.cached_characteristics, characteristic_id);
        None
    }

    fn get_gatt_characteristics_by_uuid(&mut self,
                                        adapter: &mut BluetoothAdapter,
                                        service_id: &str,
                                        characteristic_uuid: &str)
                                        -> Vec<BluetoothGATTCharacteristic> {
        let characteristics = self.get_and_cache_gatt_characteristics(adapter, service_id);
        characteristics.into_iter()
                       .filter(|c| c.get_uuid().ok() == Some(characteristic_uuid.to_string()))
                       .collect()
    }

    fn get_characteristic_properties(&self, characteristic: &BluetoothGATTCharacteristic) -> Flags {
        let mut props: Flags = Flags::empty();
        let flags = characteristic.get_flags().unwrap_or(vec!());
        for flag in flags {
            match flag.as_ref() {
                "broadcast" => props.insert(BROADCAST),
                "read" => props.insert(READ),
                "write_without_response" => props.insert(WRITE_WITHOUT_RESPONSE),
                "write" => props.insert(WRITE),
                "notify" => props.insert(NOTIFY),
                "indicate" => props.insert(INDICATE),
                "authenticated_signed_writes" => props.insert(AUTHENTICATED_SIGNED_WRITES),
                "reliable_write" => props.insert(RELIABLE_WRITE),
                "writable_auxiliaries" => props.insert(WRITABLE_AUXILIARIES),
                _ => (),
            }
        }
        props
    }

    // Descriptor

    fn get_and_cache_gatt_descriptors(&mut self,
                                      adapter: &mut BluetoothAdapter,
                                      characteristic_id: &str)
                                      -> Vec<BluetoothGATTDescriptor> {
        let descriptors = match self.get_gatt_characteristic(adapter, characteristic_id) {
            Some(c) => c.get_gatt_descriptors().unwrap_or(vec!()),
            None => vec!(),
        };

        for descriptor in &descriptors {
            self.cached_descriptors.insert(descriptor.get_id(), descriptor.clone());
            self.descriptor_to_characteristic.insert(descriptor.get_id(), characteristic_id.to_owned());
        }
        descriptors
    }

    fn get_gatt_descriptor(&mut self,
                           adapter: &mut BluetoothAdapter,
                           descriptor_id: &str)
                           -> Option<&BluetoothGATTDescriptor> {
        return_if_cached!(self.cached_descriptors, descriptor_id);
        let characteristic_id = match self.descriptor_to_characteristic.get(descriptor_id) {
            Some(c) => c.clone(),
            None => return None,
        };
        self.get_and_cache_gatt_descriptors(adapter, &characteristic_id);
        return_if_cached!(self.cached_descriptors, descriptor_id);
        None
    }

    fn get_gatt_descriptors_by_uuid(&mut self,
                                    adapter: &mut BluetoothAdapter,
                                    characteristic_id: &str,
                                    descriptor_uuid: &str)
                                    -> Vec<BluetoothGATTDescriptor> {
        let descriptors = self.get_and_cache_gatt_descriptors(adapter, characteristic_id);
        descriptors.into_iter()
                   .filter(|d| d.get_uuid().ok() == Some(descriptor_uuid.to_string()))
                   .collect()
    }

    // Methods

    fn request_device(&mut self,
                      options: RequestDeviceoptions,
                      sender: IpcSender<BluetoothResult<BluetoothDeviceMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        if let Some(ref session) = adapter.create_discovery_session().ok() {
            match session.start_discovery() {
                Ok(_) => thread::sleep(Duration::from_millis(DISCOVERY_TIMEOUT_MS)),
                //TODO: Add a new error to BluetoothError or a new static error string
                Err(err) => return drop(sender.send(Err(BluetoothError::Type(err.description().to_owned())))),
            }
            let _ = session.stop_discovery();
        }
        let devices = self.get_and_cache_devices(&mut adapter);
        let matched_devices: Vec<BluetoothDevice> = devices.into_iter()
                                                           .filter(|d| matches_filters(d, options.get_filters()))
                                                           .collect();
        if let Some(address) = self.select_device(matched_devices) {
            let device_id = match self.address_to_id.get(&address) {
                Some(id) => id.clone(),
                None => return drop(sender.send(Err(BluetoothError::NotFound))),
            };
            let mut services = options.get_services_set();
            if let Some(services_set) = self.allowed_services.get(&device_id) {
                services = services_set | &services;
            }
            self.allowed_services.insert(device_id.clone(), services);
            if let Some(device) = self.get_device(&mut adapter, &device_id) {
                let message = Ok(BluetoothDeviceMsg {
                                     id: device_id,
                                     name: device.get_name().ok(),
                                     appearance: device.get_appearance().ok(),
                                     tx_power: device.get_tx_power().ok().map(|p| p as i8),
                                     rssi: device.get_rssi().ok().map(|p| p as i8),
                                 });
                return drop(sender.send(message));
            }
        }
        return drop(sender.send(Err(BluetoothError::NotFound)));
    }

    fn gatt_server_connect(&mut self, device_id: String, sender: IpcSender<BluetoothResult<bool>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);

        match self.get_device(&mut adapter, &device_id) {
            Some(d) => {
                if d.is_connected().unwrap_or(false) {
                    return drop(sender.send(Ok(true)));
                }
                let _ = d.connect();
                for _ in 0..MAXIMUM_TRANSACTION_TIME {
                    match d.is_connected().unwrap_or(false) {
                        true => return drop(sender.send(Ok(true))),
                        false => {
                            if TESTING.load(Ordering::Relaxed) {
                                break;
                            }
                            thread::sleep(Duration::from_millis(CONNECTION_TIMEOUT_MS));
                        },
                    }
                }
                return drop(sender.send(Err(BluetoothError::Network)));
            },
            None => return drop(sender.send(Err(BluetoothError::NotFound))),
        }
    }

    fn gatt_server_disconnect(&mut self, device_id: String, sender: IpcSender<BluetoothResult<bool>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);

        match self.get_device(&mut adapter, &device_id) {
            Some(d) => {
                if !d.is_connected().unwrap_or(true) {
                    return drop(sender.send(Ok(false)));
                }
                let _ = d.disconnect();
                for _ in 0..MAXIMUM_TRANSACTION_TIME {
                    match d.is_connected().unwrap_or(true) {
                        true => thread::sleep(Duration::from_millis(CONNECTION_TIMEOUT_MS)),
                        false => return drop(sender.send(Ok(false))),
                    }
                }
                return drop(sender.send(Err(BluetoothError::Network)));
            },
            None => return drop(sender.send(Err(BluetoothError::NotFound))),
        }
    }

    fn get_primary_service(&mut self,
                           device_id: String,
                           uuid: String,
                           sender: IpcSender<BluetoothResult<BluetoothServiceMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        if !self.allowed_services.get(&device_id).map_or(false, |s| s.contains(&uuid)) {
            return drop(sender.send(Err(BluetoothError::Security)));
        }
        let services = self.get_gatt_services_by_uuid(&mut adapter, &device_id, &uuid);
        if services.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        for service in services {
            if service.is_primary().unwrap_or(false) {
                if let Ok(uuid) = service.get_uuid() {
                    return drop(sender.send(Ok(BluetoothServiceMsg {
                                                   uuid: uuid,
                                                   is_primary: true,
                                                   instance_id: service.get_id(),
                                               })));
                }
            }
        }
        return drop(sender.send(Err(BluetoothError::NotFound)));
    }

    fn get_primary_services(&mut self,
                            device_id: String,
                            uuid: Option<String>,
                            sender: IpcSender<BluetoothResult<BluetoothServicesMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let services = match uuid {
            Some(ref id) => {
                if !self.allowed_services.get(&device_id).map_or(false, |s| s.contains(id)) {
                    return drop(sender.send(Err(BluetoothError::Security)))
                }
                self.get_gatt_services_by_uuid(&mut adapter, &device_id, id)
            },
            None => self.get_and_cache_gatt_services(&mut adapter, &device_id),
        };
        if services.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        let mut services_vec = vec!();
        for service in services {
            if service.is_primary().unwrap_or(false) {
                if let Ok(uuid) = service.get_uuid() {
                    services_vec.push(BluetoothServiceMsg {
                                          uuid: uuid,
                                          is_primary: true,
                                          instance_id: service.get_id(),
                                      });
                }
            }
        }
        if services_vec.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }

        let _ = sender.send(Ok(services_vec));
    }

    fn get_included_service(&mut self,
                            service_id: String,
                            uuid: String,
                            sender: IpcSender<BluetoothResult<BluetoothServiceMsg>>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
        };
        let primary_service = match self.get_gatt_service(&mut adapter, &service_id) {
            Some(s) => s,
            None => return drop(sender.send(Err(BluetoothError::NotFound))),
        };
        let services = primary_service.get_includes().unwrap_or(vec!());
        for service in services {
            if let Ok(service_uuid) = service.get_uuid() {
                if uuid == service_uuid {
                    return drop(sender.send(Ok(BluetoothServiceMsg {
                                                   uuid: uuid,
                                                   is_primary: service.is_primary().unwrap_or(false),
                                                   instance_id: service.get_id(),
                                               })));
                }
            }
        }
        return drop(sender.send(Err(BluetoothError::NotFound)));
    }

    fn get_included_services(&mut self,
                             service_id: String,
                             uuid: Option<String>,
                             sender: IpcSender<BluetoothResult<BluetoothServicesMsg>>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
        };
        let primary_service = match self.get_gatt_service(&mut adapter, &service_id) {
            Some(s) => s,
            None => return drop(sender.send(Err(BluetoothError::NotFound))),
        };
        let services = primary_service.get_includes().unwrap_or(vec!());
        let mut services_vec = vec!();
        for service in services {
            if let Ok(service_uuid) = service.get_uuid() {
                services_vec.push(BluetoothServiceMsg {
                                      uuid: service_uuid,
                                      is_primary: service.is_primary().unwrap_or(false),
                                      instance_id: service.get_id(),
                                  });
            }
        }
        if let Some(uuid) = uuid {
            services_vec.retain(|ref s| s.uuid == uuid);
        }
        if services_vec.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }

        let _ = sender.send(Ok(services_vec));
    }

    fn get_characteristic(&mut self,
                          service_id: String,
                          uuid: String,
                          sender: IpcSender<BluetoothResult<BluetoothCharacteristicMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let characteristics = self.get_gatt_characteristics_by_uuid(&mut adapter, &service_id, &uuid);
        if characteristics.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        for characteristic in characteristics {
            if let Ok(uuid) = characteristic.get_uuid() {
                let properties = self.get_characteristic_properties(&characteristic);
                let message = Ok(BluetoothCharacteristicMsg {
                                     uuid: uuid,
                                     instance_id: characteristic.get_id(),
                                     broadcast: properties.contains(BROADCAST),
                                     read: properties.contains(READ),
                                     write_without_response: properties.contains(WRITE_WITHOUT_RESPONSE),
                                     write: properties.contains(WRITE),
                                     notify: properties.contains(NOTIFY),
                                     indicate: properties.contains(INDICATE),
                                     authenticated_signed_writes: properties.contains(AUTHENTICATED_SIGNED_WRITES),
                                     reliable_write: properties.contains(RELIABLE_WRITE),
                                     writable_auxiliaries: properties.contains(WRITABLE_AUXILIARIES),
                                 });
                return drop(sender.send(message));
            }
        }
        return drop(sender.send(Err(BluetoothError::NotFound)));
    }

    fn get_characteristics(&mut self,
                           service_id: String,
                           uuid: Option<String>,
                           sender: IpcSender<BluetoothResult<BluetoothCharacteristicsMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let characteristics = match uuid {
            Some(id) => self.get_gatt_characteristics_by_uuid(&mut adapter, &service_id, &id),
            None => self.get_and_cache_gatt_characteristics(&mut adapter, &service_id),
        };
        if characteristics.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        let mut characteristics_vec = vec!();
        for characteristic in characteristics {
            if let Ok(uuid) = characteristic.get_uuid() {
                let properties = self.get_characteristic_properties(&characteristic);
                characteristics_vec.push(
                                BluetoothCharacteristicMsg {
                                    uuid: uuid,
                                    instance_id: characteristic.get_id(),
                                    broadcast: properties.contains(BROADCAST),
                                    read: properties.contains(READ),
                                    write_without_response: properties.contains(WRITE_WITHOUT_RESPONSE),
                                    write: properties.contains(WRITE),
                                    notify: properties.contains(NOTIFY),
                                    indicate: properties.contains(INDICATE),
                                    authenticated_signed_writes: properties.contains(AUTHENTICATED_SIGNED_WRITES),
                                    reliable_write: properties.contains(RELIABLE_WRITE),
                                    writable_auxiliaries: properties.contains(WRITABLE_AUXILIARIES),
                                });
            }
        }
        if characteristics_vec.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }

        let _ = sender.send(Ok(characteristics_vec));
    }

    fn get_descriptor(&mut self,
                      characteristic_id: String,
                      uuid: String,
                      sender: IpcSender<BluetoothResult<BluetoothDescriptorMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let descriptors = self.get_gatt_descriptors_by_uuid(&mut adapter, &characteristic_id, &uuid);
        if descriptors.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        for descriptor in descriptors {
            if let Ok(uuid) = descriptor.get_uuid() {
                return drop(sender.send(Ok(BluetoothDescriptorMsg {
                                               uuid: uuid,
                                               instance_id: descriptor.get_id(),
                                           })));
            }
        }
        return drop(sender.send(Err(BluetoothError::NotFound)));
    }

    fn get_descriptors(&mut self,
                       characteristic_id: String,
                       uuid: Option<String>,
                       sender: IpcSender<BluetoothResult<BluetoothDescriptorsMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let descriptors = match uuid {
            Some(id) => self.get_gatt_descriptors_by_uuid(&mut adapter, &characteristic_id, &id),
            None => self.get_and_cache_gatt_descriptors(&mut adapter, &characteristic_id),
        };
        if descriptors.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        let mut descriptors_vec = vec!();
        for descriptor in descriptors {
            if let Ok(uuid) = descriptor.get_uuid() {
                descriptors_vec.push(BluetoothDescriptorMsg {
                                         uuid: uuid,
                                         instance_id: descriptor.get_id(),
                                     });
            }
        }
        if descriptors_vec.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        let _ = sender.send(Ok(descriptors_vec));
    }

    fn read_value(&mut self, id: String, sender: IpcSender<BluetoothResult<Vec<u8>>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let mut value = self.get_gatt_characteristic(&mut adapter, &id)
                            .map(|c| c.read_value().unwrap_or(vec![]));
        if value.is_none() {
            value = self.get_gatt_descriptor(&mut adapter, &id)
                        .map(|d| d.read_value().unwrap_or(vec![]));
        }
        let _ = sender.send(value.ok_or(BluetoothError::NotSupported));
    }

    fn write_value(&mut self, id: String, value: Vec<u8>, sender: IpcSender<BluetoothResult<bool>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let mut result = self.get_gatt_characteristic(&mut adapter, &id)
                             .map(|c| c.write_value(value.clone()));
        if result.is_none() {
            result = self.get_gatt_descriptor(&mut adapter, &id)
                         .map(|d| d.write_value(value.clone()));
        }
        let message = match result {
            Some(v) => match v {
                Ok(_) => Ok(true),
                Err(_) => return drop(sender.send(Err(BluetoothError::NotSupported))),
            },
            None => return drop(sender.send(Err(BluetoothError::NotSupported))),
        };
        let _ = sender.send(message);
    }
}
