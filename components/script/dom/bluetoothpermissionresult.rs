/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::BluetoothResponse;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding::{self, BluetoothPermissionResultMethods};
use dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorBinding::NavigatorMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::{AsyncBluetoothListener, Bluetooth};
use dom::bluetooth::{OPTIONS_ERROR, response_async};
use dom::bluetoothdevice::BluetoothDevice;
use dom::globalscope::GlobalScope;
use dom::permissionstatus::{PermissionStatus, PermissionDescriptorType};
use dom::promise::Promise;
use js::jsapi::JSContext;
use std::rc::Rc;

const DESCRIPTOR_TYPE_ERROR: &'static str = "Wrong type of descriptor in argument list.";

#[dom_struct]
pub struct BluetoothPermissionResult {
    parent: PermissionStatus,
    devices: DOMRefCell<Vec<JS<BluetoothDevice>>>,
}

impl BluetoothPermissionResult {
    pub fn new_inherited() -> BluetoothPermissionResult {
        BluetoothPermissionResult {
            parent: PermissionStatus::new_inherited(),
            devices: DOMRefCell::new(Vec::new()),
        }
    }

    pub fn new(global: &GlobalScope) -> Root<BluetoothPermissionResult> {
        reflect_dom_object(box BluetoothPermissionResult::new_inherited(),
                           global,
                           BluetoothPermissionResultBinding::Wrap)
    }

    fn get_bluetooth(&self) -> Root<Bluetooth> {
        self.global().as_window().Navigator().Bluetooth()
    }

    pub fn create_from_descriptor(global: &GlobalScope,
                                  descriptor: PermissionDescriptorType)
                                  -> Root<BluetoothPermissionResult> {
        let bt_permission_result = BluetoothPermissionResult::new(global);
        bt_permission_result.upcast::<PermissionStatus>().set_query(descriptor);
        bt_permission_result
    }

    pub fn permission_request(&self, promise: &Rc<Promise>) -> ErrorResult {
        let descriptor = self.parent.get_query().borrow();
        let options = match *descriptor {
            PermissionDescriptorType::Bluetooth(ref d) => d,
            _ => return Err(Error::Type(DESCRIPTOR_TYPE_ERROR.to_owned())),
        };

        // Step 1.
        if (options.filters.is_some() && options.acceptAllDevices) ||
           (options.filters.is_none() && !options.acceptAllDevices) {
            return Err(Error::Type(OPTIONS_ERROR.to_owned()));
        }

        // Step 2.
        let sender = response_async(promise, self);
        let bluetooth = self.get_bluetooth();
        Ok(bluetooth.request_bluetooth_devices(promise, sender, &options.filters, &options.optionalServices))
    }
}


// https://w3c.github.io/permissions/#dom-permissions-request
// TODO: Make a BluetoothResponse variant wich returns with a vector of devices that matches.
impl AsyncBluetoothListener for BluetoothPermissionResult {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        match response {
            BluetoothResponse::RequestDevice(device) => {
                let bluetooth = &self.get_bluetooth();
                let mut device_instance_map = bluetooth.get_device_map().borrow_mut();
                if let Some(existing_device) = device_instance_map.get(&device.id.clone()) {
                    *self.devices.borrow_mut() = vec!(JS::from_ref(&**existing_device));
                    // Step 8.
                    return promise.resolve_native(promise_cx, &self.parent);
                }
                let bt_device = BluetoothDevice::new(&self.global(),
                                                     DOMString::from(device.id.clone()),
                                                     device.name.map(DOMString::from),
                                                     bluetooth);
                device_instance_map.insert(device.id, JS::from_ref(&bt_device));
                *self.devices.borrow_mut() = vec!(JS::from_ref(&bt_device));
                // Step 8.
                promise.resolve_native(promise_cx, &self.parent);
            },
            _ => promise.reject_error(promise_cx, Error::Type("Something went wrong...".to_owned())),
        }
    }
}

impl BluetoothPermissionResultMethods for BluetoothPermissionResult {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothpermissionresult-devices
    fn Devices(&self) -> Vec<Root<BluetoothDevice>> {
        let device_vec: Vec<Root<BluetoothDevice>> =
            self.devices.borrow().iter().map(|d| Root::from_ref(&**d)).collect();
        device_vec
    }
}
