/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::PermissionStatusBinding::{self, PermissionStatusMethods, PermissionState};
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;

#[dom_struct]
pub struct PermissionStatus {
    eventtarget: EventTarget,
    permission_state: PermissionState,
}

impl PermissionStatus {
    pub fn new_inherited(permission_state: PermissionState) -> PermissionStatus {
        PermissionStatus {
            eventtarget: EventTarget::new_inherited(),
            permission_state: permission_state,
        }
    }

    pub fn new(global: &GlobalScope, permission_state: PermissionState) -> Root<PermissionStatus> {
        reflect_dom_object(box PermissionStatus::new_inherited(permission_state),
                           global,
                           PermissionStatusBinding::Wrap)
    }
}

impl PermissionStatusMethods for PermissionStatus {

    // https://w3c.github.io/permissions/#dom-permissionstatus-state
    fn State(&self) -> PermissionState {
        self.permission_state.clone()
    }

    // https://w3c.github.io/permissions/#dom-permissionstatus-onchange
     event_handler!(onchange, GetOnchange, SetOnchange);
}
