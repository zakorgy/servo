/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PermissionStatusBinding::{self, PermissionStatusMethods, PermissionState};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::eventtarget::EventTarget;

#[dom_struct]
pub struct PermissionStatus {
    eventtarget: EventTarget,
    state: PermissionState,
}

impl PermissionStatus {
    pub fn new_inherited(state: &PermissionState) -> PermissionStatus {
        PermissionStatus {
            eventtarget: EventTarget::new_inherited(),
            state: state.clone(),
        }
    }

    pub fn new(global: GlobalRef, state: &PermissionState) -> Root<PermissionStatus> {
        reflect_dom_object(box PermissionStatus::new_inherited(state),
                           global,
                           PermissionStatusBinding::Wrap)
    }

    pub fn update_state(&mut self, descriptor: &PermissionDescriptor) {
        retrieve_permission_storage(&descriptor.name, self.eventtarget.global().r())
    }
}

impl PermissionStatusMethods for PermissionStatus {

    fn State(&self) -> PermissionState {
        self.state.clone()
    }

    event_handler!(onchange, GetOnchange, SetOnchange);
}