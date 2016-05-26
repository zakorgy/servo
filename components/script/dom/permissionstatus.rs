/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PermissionStatusBinding::{self, PermissionStatusMethods, PermissionState, PermissionStorage};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::eventtarget::EventTarget;
use dom::permissions::{PERMISSION_STORE, PermissionStore};
use dom::bindings::codegen::Bindings::PermissionsBinding::PermissionDescriptor;

#[dom_struct]
pub struct PermissionStatus {
    eventtarget: EventTarget,
    state: PermissionState,
}

// https://w3c.github.io/permissions/#permission
/*pub trait PermissionAlgorithms {
    // https://w3c.github.io/permissions/#permission-query-algorithm
    fn query<D, S>(&mut self, storage: S, desc: Option<&D>) -> ();
    // https://w3c.github.io/permissions/#permission-request-algorithm
    //fn request<D, S>(&mut self, desc: D, storage: S) -> Option<R> where R: PermissionStatusMethods;
    // https://w3c.github.io/permissions/#permission-revocation-algorithm
    //fn revocation(&self);
}

impl PermissionAlgorithms for PermissionStatus {
    fn query<PermissionDescriptor, PermissionStorage>(&mut self, 
                                                      storage: PermissionStorage,
                                                      desc: Option<&PermissionDescriptor>) {
        self.state = storage.state;
    }
}*/

impl PermissionStatus {
    pub fn new_inherited() -> PermissionStatus {
        PermissionStatus {
            eventtarget: EventTarget::new_inherited(),
            state: PermissionState::Denied,
        }
    }

    pub fn new(global: GlobalRef) -> Root<PermissionStatus> {
        reflect_dom_object(box PermissionStatus::new_inherited(),
                           global,
                           PermissionStatusBinding::Wrap)
    }

    // https://w3c.github.io/permissions/#update-the-state
    pub fn update_state(&mut self, descriptor: &PermissionDescriptor) {
        // Step 1
        let mut storage = PermissionStorage { state: PermissionState::Denied, };
        PERMISSION_STORE.with(|pstore|{
            storage = pstore.borrow()
                            .retrieve_permission_storage(&descriptor.name, self.eventtarget.global().r());
        });
        // Step 2 missing maybe implement a trait for the permission functions
        // permission_query(descriptor, storage, self)
        // https://w3c.github.io/permissions/#boolean-permission-query-algorithm
        self.state = storage.state;
    }
}

impl PermissionStatusMethods for PermissionStatus {
    // https://w3c.github.io/permissions/#dom-permissionstatus-state
    fn State(&self) -> PermissionState {
        self.state.clone()
    }
    // https://w3c.github.io/permissions/#dom-permissionstatus-onchange
    event_handler!(onchange, GetOnchange, SetOnchange);
}