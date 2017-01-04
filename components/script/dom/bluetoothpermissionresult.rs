/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothRequest, BluetoothResponse};
use bluetooth_traits::scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding;
use dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding::BluetoothPermissionDescriptor;
use dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding::BluetoothPermissionResultMethods;
use dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorBinding::NavigatorMethods;
use dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionState;
use dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionStatusBinding::PermissionStatusMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{reflect_dom_object, DomObject};
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::bluetooth::{AsyncBluetoothListener, Bluetooth};
use dom::bluetooth::{canonicalize_filter, get_allowed_devices, response_async, OPTIONS_ERROR};
use dom::bluetoothdevice::BluetoothDevice;
use dom::globalscope::GlobalScope;
use dom::permissionstatus::{PermissionStatus, PermissionDescriptorType};
use dom::promise::Promise;
use heapsize::HeapSizeOf;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{JSContext, JSTracer};
use std::rc::Rc;

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

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }

    pub fn create_from_descriptor(global: &GlobalScope,
                              descriptor: PermissionDescriptorType)
                              -> Root<BluetoothPermissionResult> {
        let bt_permission_result = BluetoothPermissionResult::new(global);
        bt_permission_result.upcast::<PermissionStatus>().set_query(descriptor);
        bt_permission_result
    }

    #[allow(unsafe_code)]
    #[allow(unrooted_must_root)]
    pub unsafe fn permission_query(&self,
                                   permission_desc: &PermissionDescriptorType,
                                   promise: &Rc<Promise>,
                                   cx: *mut JSContext) {
        let desc = match permission_desc {
            &PermissionDescriptorType::Bluetooth(ref d) => d,
            _ => panic!("Wrong type of descriptor in argument list."),
        };

        // Step 2.
        self.parent.permission_query(permission_desc);

        // Step 3.
        if let PermissionState::Denied = self.parent.State() {
            *self.devices.borrow_mut() = Vec::new();
            return promise.resolve_native(cx, &self.parent);;
        }

        // Step 4.
        let mut matching_devices: Vec<JS<BluetoothDevice>>  = Vec::new();

        // TODO: Step 5: Create a map between the current setting object and BluetoothPermissionData
        // extra permission data, which replaces the exisitng EXTRA_PERMISSION_DATA global variable.
        // For this also use the extra permission data constraints from the specification:
        // https://webbluetoothcg.github.io/web-bluetooth/#dictdef-bluetoothpermissiondata

        let bluetooth = self.get_bluetooth();
        let device_map = bluetooth.get_device_map().borrow();
        let mut device_id = String::new();

        // Step 5.
        let allowed_devices = get_allowed_devices();

        // Step 6.
        for allowed_device in allowed_devices {
            // Step 6.1.
            if let Some(id) = desc.deviceId.clone() {
                if allowed_device.deviceId != id {
                    continue;
                } else {
                    device_id = String::from(id);
                }
            }

            // Step 6.2.
            // Instead of creating an internal slot we send an ipc message to the Bluetooth thread
            // to check if one of the filters matches.
            if let Some(ref filters) = desc.filters {
                let mut scan_filters: Vec<BluetoothScanfilter> = Vec::new();
                // TODO: Create an issue for the spec, to make the canonicalization step here.
                for filter in filters {
                    match canonicalize_filter(&filter) {
                        Ok(f) => scan_filters.push(f),
                        Err(err) => {
                            promise.reject_error(cx, err);
                            return;
                        },
                    }
                }
                let (sender, receiver) = ipc::channel().unwrap();
                self.get_bluetooth_thread()
                    .send(BluetoothRequest::MatchesFilter(BluetoothScanfilterSequence::new(scan_filters),
                                                          device_id.clone(),
                                                          sender))
                    .unwrap();

                match receiver.recv().unwrap() {
                    Ok(true) => {},
                    Ok(false) => continue,
                    Err(err) => {
                        promise.reject_error(cx, Error::from(err));
                        return;
                    },
                }
            }

            // Step 6.4.
            // TODO: Implement this correctly, not just using device ids here.
            // https://webbluetoothcg.github.io/web-bluetooth/#get-the-bluetoothdevice-representing
            if let Some(ref device) = device_map.get(&device_id) {
                matching_devices.push(JS::from_ref(&**device.clone()));
            }
        }

        // Step 7.
        *self.devices.borrow_mut() = matching_devices;

        // https://w3c.github.io/permissions/#dom-permissions-query
        // Step 7.
        promise.resolve_native(cx, &self.parent);
    }

    #[allow(unsafe_code)]
    pub unsafe fn permission_request(&self,
                                 permission_desc: &PermissionDescriptorType,
                                 promise: &Rc<Promise>)
                                 -> ErrorResult {
         let options = match permission_desc {
             &PermissionDescriptorType::Bluetooth(ref d) => d,
             _ => panic!("Wrong type of descriptor in argument list."),
         };

         // Step 1.
         if (options.filters.is_some() && options.acceptAllDevices) ||
            (options.filters.is_none() && !options.acceptAllDevices) {
             // NOTE: Maybe reject promise here?
             return Err(Error::Type(OPTIONS_ERROR.to_owned()));
         }

         // Step 2.
         let sender = response_async(promise, self);
         let bluetooth = self.get_bluetooth();
         return Ok(bluetooth.request_bluetooth_devices(promise, sender, &options.filters, &options.optionalServices));
    }
}

impl HeapSizeOf for BluetoothPermissionDescriptor {
    fn heap_size_of_children(&self) -> usize {
        self.parent.heap_size_of_children() +
        self.acceptAllDevices.heap_size_of_children() +
        self.deviceId.heap_size_of_children()// +
        // TODO: Implement heap_size_of for these two
        // self.filters.heap_size_of_children() +
        // self.optionalServices.heap_size_of_children()
    }
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for BluetoothPermissionDescriptor {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.parent.trace(trc);
        self.acceptAllDevices.trace(trc);
        self.deviceId.trace(trc);
        // TODO: Implement trace for these two
        // self.filters.trace(trc);
        // self.optionalServices.trace(trc);
    }
}

// TODO: Make a BluetoothResponse variant wich returns with a vector of devices that matches .
// TODO: Add step annotations.
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
