/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_scanfilter::BluetoothScanfilterSequence;
use rand::{self, Rng};
use rustc_serialize::base64::{ToBase64, STANDARD};
use std::collections::{HashMap, HashSet};

type DeviceAddressToIdMap = HashMap<String, String>;
type DeviceIdToAddressMap = HashMap<String, String>;
//type DeviceAdressToServicesMap = HashMap<String, HashSet<String>>;
type DeviceAdressToServicesMap = HashMap<String, HashSet<String>>;

pub struct AllowedDevicesMap {
    // after we have the proper origin handling we can replace the Option<String> with String
    origin_to_device_address_to_id_map: HashMap<Option<String>, DeviceAddressToIdMap>,
    // origin_to_device_id_to_address_map: HashMap<Option<String>, DeviceIdToAddressMap>,
    origin_to_device_adress_to_services_map: HashMap<Option<String>, DeviceAdressToServicesMap>,
    //origin_to_device_id_to_services_map: HashMap<Option<String>, DeviceIdToServicesMap>,
    device_id_set: HashSet<String>,
}


fn add_union_of_services_to(filters: &BluetoothScanfilterSequence,
                            optional_services: Vec<String>,
                            union_of_services: &mut HashSet<String>) {
    for filter in filters.iter() {
        for uuid in filter.get_services() {
            union_of_services.insert(uuid.clone());
        }
    }
    for uuid in optional_services {
        union_of_services.insert(uuid);
    }
}

impl AllowedDevicesMap {

    fn generate_device_id(&self) -> String {
        let mut device_id = String::new();
        while self.device_id_set.contains(&device_id) {
            let mut v = vec![0u8; 16];
            rand::thread_rng().fill_bytes(&mut v);
            device_id = v.to_base64(STANDARD);
        }
        device_id
    }

    pub fn new() -> AllowedDevicesMap {
        AllowedDevicesMap {
            origin_to_device_address_to_id_map: HashMap::new(),
            // origin_to_device_id_to_address_map: HashMap::new(),
            origin_to_device_adress_to_services_map: HashMap::new(),
            // origin_to_device_id_to_services_map: HashMap::new(),
            device_id_set: HashSet::new(),
        }
    }

    pub fn add_device(&mut self,
                      origin: &Option<String>,
                      device_adress: &str,
                      filters: &BluetoothScanfilterSequence,
                      optional_services: Vec<String>)
                      ->  Result<String, String> {
        // TODO: check if the origin is trustworthy
        if self.origin_to_device_address_to_id_map.contains_key(origin) {
            let device_adress_to_id_map = {
                self.origin_to_device_address_to_id_map.get_mut(origin).map_or(HashMap::new(),|m| m.clone())
            };
            if let Some(device_id) = device_adress_to_id_map.get(device_adress) {
                add_union_of_services_to(filters,
                                         optional_services,
                                         self.origin_to_device_adress_to_services_map
                                             .get_mut(origin).unwrap()
                                             .get_mut(device_id).unwrap());
                Ok(device_id.clone())
            } else {
                Err(String::from("valami"))
            }
        } else {
            // If the 'origin_to_device_address_to_id_map' does not contain the origin the other 2 map
            // does not contain it too. So we need to insert the new values into them.
            // self.origin_to_device_id_to_address_map.insert(origin.clone(), HashMap::new());
            self.origin_to_device_address_to_id_map.insert(origin.clone(), HashMap::new());
            self.origin_to_device_adress_to_services_map.insert(origin.clone(), HashMap::new());
            let device_id = self.generate_device_id();
            // self.origin_to_device_id_to_address_map.get_mut(origin)
            //                                        .map(|m| m.insert(device_id.clone(), device_adress.to_string()));
            self.origin_to_device_address_to_id_map.get_mut(origin)
                                                   .map(|m| m.insert(device_adress.to_string(), device_id.clone()));
            self.origin_to_device_adress_to_services_map.get_mut(origin)
                                                    .map(|m| m.insert(device_adress.to_string(), HashSet::new()));
            add_union_of_services_to(filters,
                                     optional_services,
                                     self.origin_to_device_adress_to_services_map
                                         .get_mut(origin).unwrap()
                                         .get_mut(device_adress).unwrap());
            Ok(device_id)
        }
    }

    /*pub fn remove_device(&mut self,
                         origin: &Option<String>,
                         device_adress: &str) {
        let device_id = self.get_device_id(origin, device_adress).clone();
        if device_id.is_empty() {
            return;
        }

        self.origin_to_device_address_to_id_map.get_mut(origin).map(|m| m.remove(device_adress));
        // self.origin_to_device_id_to_address_map.get_mut(origin).map(|m| m.remove(&device_id));
        self.origin_to_device_adress_to_services_map.get_mut(origin).map(|m| m.remove(device_adress));

        if self.origin_to_device_address_to_id_map.get(origin).map(|m| m.is_empty()) == Some(true) {
            self.origin_to_device_address_to_id_map.remove(origin);
            // self.origin_to_device_id_to_address_map.remove(origin);
            self.origin_to_device_adress_to_services_map.remove(origin);
        }
    }

    pub fn get_device_id(&self,
                         origin: &Option<String>,
                         device_adress: &str)
                         -> String {
        if let Some(device_adress_to_id_map) = self.origin_to_device_address_to_id_map.get(origin) {
            if let Some(id) = device_adress_to_id_map.get(device_adress) {
                return id.clone();
            } else {
                return String::from("");
            }
        } else {
            return String::from("");
        }
    }

    pub fn get_device_adress(&self,
                             origin: &Option<String>,
                             device_id: &str)
                             -> String {
        if let Some(device_id_to_adress_map) = self.origin_to_device_id_to_address_map.get(origin) {
            if let Some(address) = device_id_to_adress_map.get(device_id) {
                return address.clone();
            }else {
                return String::from("");
            }
        } else {
            return String::from("");
        }
    }*/

    pub fn is_origin_allowed_to_acces_service(&self,
                                              origin: Option<String>,
                                              //device_id: &str,
                                              device_adress: &str,
                                              service_uuid: &str)
                                              -> bool {
        if let Some(device_id_to_services_map) = self.origin_to_device_adress_to_services_map.get(&origin) {
            if let Some(services_set) = device_id_to_services_map.get(device_adress) {
                return services_set.contains(service_uuid)
            } else {
                false
            }
        } else {
            false
        }
    }
}
