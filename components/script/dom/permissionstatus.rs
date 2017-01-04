/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::clone::Clone;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding::BluetoothPermissionDescriptor;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::PermissionStatusBinding::{self, PermissionStatusMethods};
use dom::bindings::codegen::Bindings::PermissionStatusBinding::{PermissionState, PermissionDescriptor};
use dom::bindings::error::ErrorResult;
use dom::bindings::js::Root;
use dom::bindings::reflector::{reflect_dom_object, DomObject};
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::permissions::get_descriptors_permission_state;

#[derive(Clone, JSTraceable, HeapSizeOf)]
pub enum PermissionDescriptorType {
    // TODO(zakorgy): Finish this list.
    Undefined,
    Default(PermissionDescriptor),
    Bluetooth(BluetoothPermissionDescriptor),
}

#[dom_struct]
pub struct PermissionStatus {
    eventtarget: EventTarget,
    state: DOMRefCell<PermissionState>,
    query: DOMRefCell<PermissionDescriptorType>,
}

impl PermissionStatus {
    pub fn new_inherited() -> PermissionStatus {
        PermissionStatus {
            eventtarget: EventTarget::new_inherited(),
            state: DOMRefCell::new(PermissionState::Denied),
            query: DOMRefCell::new(PermissionDescriptorType::Undefined),
        }
    }

    pub fn new(global: &GlobalScope) -> Root<PermissionStatus> {
        reflect_dom_object(box PermissionStatus::new_inherited(),
                           global,
                           PermissionStatusBinding::Wrap)
    }

    pub fn set_query(&self, permission_descriptor: PermissionDescriptorType) {
        *self.query.borrow_mut() = permission_descriptor;
    }

    pub fn get_query(&self) -> &DOMRefCell<PermissionDescriptorType> {
        &self.query
    }

    // https://w3c.github.io/permissions/#create-a-permissionstatus
    pub fn create_from_descriptor(global: &GlobalScope,
                              permission_desc: PermissionDescriptorType)
                              -> Root<PermissionStatus> {
        let permission_status = PermissionStatus::new(global);
        permission_status.set_query(permission_desc);
        permission_status
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#boolean-permission-query-algorithm
    pub unsafe fn permission_query(&self, permission_desc: &PermissionDescriptorType) {
        // Step 1.
        *self.state.borrow_mut() =
            get_descriptors_permission_state(permission_desc, self.eventtarget.reflector().get_jsobject().get());
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#boolean-permission-request-algorithm
    pub unsafe fn permission_request(&self, permission_desc: &PermissionDescriptorType) -> ErrorResult {
        // Step 1.
        self.permission_query(permission_desc);
        // Step 2.
        match *self.state.borrow() {
            PermissionState::Prompt => {
                // TODO: Step 3: Ask the users's permission.
                // https://w3c.github.io/permissions/#request-permission-to-use

                // Step 4.
                return Ok(self.permission_query(permission_desc));
            },
            _ => {
                return Ok(());
            },
        }
    }
}

impl Clone for PermissionDescriptor {
    fn clone(&self) -> PermissionDescriptor {
        PermissionDescriptor {
            name: self.name.clone(),
        }
    }
}

impl Clone for BluetoothPermissionDescriptor {
    fn clone(&self) -> BluetoothPermissionDescriptor {
        BluetoothPermissionDescriptor {
            parent: self.parent.clone(),
            acceptAllDevices: self.acceptAllDevices.clone(),
            deviceId: self.deviceId.clone(),
            filters: self.filters.clone(),
            optionalServices: self.optionalServices.clone(),
        }
    }
}

impl PermissionStatusMethods for PermissionStatus {
    // https://w3c.github.io/permissions/#dom-permissionstatus-state
    fn State(&self) -> PermissionState {
        self.state.borrow().clone()
    }

    // https://w3c.github.io/permissions/#dom-permissionstatus-onchange
     event_handler!(onchange, GetOnchange, SetOnchange);
}
