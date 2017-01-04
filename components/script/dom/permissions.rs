/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PermissionsBinding::{self, PermissionsMethods};
use dom::bindings::js::Root;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use js::jsapi::{JSObject, JSContext};
use std::rc::Rc;

#[dom_struct]
pub struct Permissions {
    reflector_: Reflector,
}

impl Permissions {
    pub fn new_inherited() -> Permissions {
        Permissions {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> Root<Permissions> {
        reflect_dom_object(box Permissions::new_inherited(),
                           global,
                           PermissionsBinding::Wrap)
    }
}

impl PermissionsMethods for Permissions {

    // https://w3c.github.io/permissions/#dom-permissions-query
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn Query(&self, _cx: *mut JSContext, _permissionDesc: *mut JSObject) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        p
    }

    // https://w3c.github.io/permissions/#dom-permissions-request
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn Request(&self, _cx: *mut JSContext, _permissionDesc: *mut JSObject) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        p
    }

    // https://w3c.github.io/permissions/#dom-permissions-revoke
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn Revoke(&self, _cx: *mut JSContext, _permissionDesc: *mut JSObject) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        p
    }
}
