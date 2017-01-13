/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding::{self, BluetoothPermissionResultMethods};
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::bluetoothdevice::BluetoothDevice;
use dom::globalscope::GlobalScope;
use dom::permissionstatus::PermissionStatus;

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
}

impl BluetoothPermissionResultMethods for BluetoothPermissionResult {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothpermissionresult-devices
    fn Devices(&self) -> Vec<Root<BluetoothDevice>> {
        let device_vec: Vec<Root<BluetoothDevice>> =
            self.devices.borrow().iter().map(|d| Root::from_ref(&**d)).collect();
        device_vec
    }
}
