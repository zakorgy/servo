/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding::BluetoothPermissionDescriptor;
use dom::bindings::codegen::Bindings::PermissionStatusBinding::{PermissionDescriptor, PermissionName, PermissionState};
use dom::bindings::codegen::Bindings::PermissionsBinding::{self, PermissionsMethods};
use dom::bindings::error::Error;
use dom::bindings::js::Root;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::trace::JSTraceable;
use dom::bluetoothpermissionresult::BluetoothPermissionResult;
use dom::globalscope::GlobalScope;
use dom::permissionstatus::{PermissionStatus, PermissionDescriptorType};
use dom::promise::Promise;
use heapsize::HeapSizeOf;
use js::conversions::{ConversionResult, FromJSValConvertible};
use js::jsapi::{JS_GetIsSecureContext, JSContext, JSObject, JSTracer};
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::get_object_compartment;
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

fn allowed_in_nonsecure_contexts(permission_name: PermissionName) -> bool {
    match permission_name {
        // https://w3c.github.io/permissions/#geolocation
        PermissionName::Geolocation => true,
        // https://w3c.github.io/permissions/#notifications
        PermissionName::Notifications => true,
        // https://w3c.github.io/permissions/#push
        PermissionName::Push => false,
        // https://w3c.github.io/permissions/#midi
        PermissionName::Midi => true,
        // https://w3c.github.io/permissions/#media-devices
        PermissionName::Camera => false,
        PermissionName::Microphone => false,
        PermissionName::Speaker => false,
        PermissionName::Device_info => false,
        // https://w3c.github.io/permissions/#background-sync
        PermissionName::Background_sync => false,
        // https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices Step 4.
        PermissionName::Bluetooth => false,
        // https://storage.spec.whatwg.org/#dom-permissionname-persistent-storage
        PermissionName::Persistent_storage => false,
    }
}

#[allow(unsafe_code)]
unsafe fn create_root_descriptor(cx: *mut JSContext,
                                permission_descriptor_obj: *mut JSObject)
                                -> Result<PermissionDescriptor, String> {
     rooted!(in(cx) let mut property = UndefinedValue());
     let jsval = ObjectValue(permission_descriptor_obj);
     property.handle_mut().set(jsval);
     match PermissionDescriptor::from_jsval(cx, property.handle(), ()) {
         Ok(ConversionResult::Success(descriptor)) => Ok(descriptor),
         Ok(ConversionResult::Failure(message)) => Err(String::from(message)),
         Err(_) => Err(String::from("Can not convert to IDL value of type PermissionDescriptor")),
     }
}

#[allow(unsafe_code)]
unsafe fn create_bluetooth_descriptor(cx: *mut JSContext,
                                      permission_descriptor_obj: *mut JSObject)
                                      -> Result<BluetoothPermissionDescriptor, String> {
     rooted!(in(cx) let mut property = UndefinedValue());
     let jsval = ObjectValue(permission_descriptor_obj);
     property.handle_mut().set(jsval);
     match BluetoothPermissionDescriptor::from_jsval(cx, property.handle(), ()) {
         Ok(ConversionResult::Success(descriptor)) => Ok(descriptor),
         Ok(ConversionResult::Failure(message)) => Err(String::from(message)),
         Err(_) => Err(String::from("Can not convert to IDL value of type BluetoothPermissionDescriptor")),
     }
}

#[allow(unsafe_code)]
// https://w3c.github.io/permissions/#permission-state
pub unsafe fn get_descriptors_permission_state(descriptor: &PermissionDescriptorType,
                                               /*TODO: make this optional*/obj: *mut JSObject)
                                               -> PermissionState {
   let compartment = get_object_compartment(obj);
    // TODO: Step 1.

    let name = match descriptor {
        // TODO(zakorgy): Finish this list.
        &PermissionDescriptorType::Default(ref desc) => desc.name,
        &PermissionDescriptorType::Bluetooth(ref desc) => desc.parent.name,
        &PermissionDescriptorType::Undefined => return PermissionState::Denied,
    };

    // Step 2.
    // TODO: Check the correctness of JS_GetIsSecureContext
    if !(JS_GetIsSecureContext(compartment) || allowed_in_nonsecure_contexts(name)) {
        return PermissionState::Denied;
    }
    // TODO: Step 3: Store the invocation results
    // TODO: Step 4: Make interaction with the user to discover the user's intent.
    PermissionState::Granted
}

impl HeapSizeOf for PermissionDescriptor {
    fn heap_size_of_children(&self) -> usize {
        self.name.heap_size_of_children()
    }
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for PermissionDescriptor {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.name.trace(trc);
    }
}

#[allow(unsafe_code)]
// https://w3c.github.io/permissions/#dom-permissions-query
unsafe fn sync_default_permission_query_call(global: &GlobalScope,
                                             descriptor: PermissionDescriptor,
                                             promise: &Rc<Promise>,
                                             cx: *mut JSContext) {
    // Step 5.
    let status = PermissionStatus::create_from_descriptor(global, PermissionDescriptorType::Default(descriptor));
    // Step 6.
    status.permission_query(&*status.get_query().borrow());
    // Step 7.
    promise.resolve_native(cx, &status);
}

#[allow(unsafe_code)]
// https://w3c.github.io/permissions/#dom-permissions-request
unsafe fn sync_default_permission_request_call(global: &GlobalScope,
                                               descriptor: PermissionDescriptor,
                                               promise: &Rc<Promise>,
                                               cx: *mut JSContext) {
    // Step 5.
    let status = PermissionStatus::create_from_descriptor(global, PermissionDescriptorType::Default(descriptor));
    // Step 6.
    // NOTE: `status.query` is the same as descriptor
    if let Err(err) = status.permission_request(&*status.get_query().borrow()) {
        // Step 7.
        promise.reject_error(cx, err);
    }
    // Step 8.
    promise.resolve_native(cx, &status);
}

#[allow(unsafe_code)]
// https://w3c.github.io/permissions/#dom-permissions-query
unsafe fn bluetooth_permission_query_call(global: &GlobalScope,
                                          descriptor: BluetoothPermissionDescriptor,
                                          promise: &Rc<Promise>,
                                          cx: *mut JSContext) {
    // Step 5.
    let result = BluetoothPermissionResult::create_from_descriptor(
                    global, PermissionDescriptorType::Bluetooth(descriptor.clone()));
    // Step 6.
    result.permission_query(&PermissionDescriptorType::Bluetooth(descriptor), &promise, cx);
}

#[allow(unrooted_must_root)]
#[allow(unsafe_code)]
// https://w3c.github.io/permissions/#dom-permissions-request
unsafe fn bluetooth_permission_request_call(global: &GlobalScope,
                                            descriptor: BluetoothPermissionDescriptor,
                                            promise: &Rc<Promise>,
                                            cx: *mut JSContext) {
    // Step 5.
    let result = BluetoothPermissionResult::create_from_descriptor(
                    global, PermissionDescriptorType::Bluetooth(descriptor.clone()));
    // Step 6.
    // NOTE: `result.paraent.query` is the same as descriptor
    if let Err(err) = result.permission_request(&PermissionDescriptorType::Bluetooth(descriptor), &promise) {
        // Step 7.
        promise.reject_error(cx, err);
    }
}

impl PermissionsMethods for Permissions {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#dom-permissions-query
    unsafe fn Query(&self, cx: *mut JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        // Step 3.
        let p = Promise::new(&self.global());

        // Step 1.
        let root_desc = match create_root_descriptor(cx, permissionDesc) {
            Ok(descriptor) => descriptor,
            Err(message) => {
                p.reject_error(cx, Error::Type(message));
                return p;
            },
        };

        // Step 2.
        match root_desc.name {
            // TODO: Add support for not default cases
            PermissionName::Bluetooth => {
                let type_desc = match create_bluetooth_descriptor(cx, permissionDesc) {
                    Ok(descriptor) => descriptor,
                    Err(message) => {
                        p.reject_error(cx, Error::Type(message));
                        return p;
                    },
                };
                bluetooth_permission_query_call(&self.global(), type_desc, &p, cx);
            },
            _ => {
                // Step 4: TODO: Add an async implementation instead of sync_default_permission_query_call
                sync_default_permission_query_call(&self.global(), root_desc, &p, cx);
            },
        };

        return p;
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#dom-permissions-request
    unsafe fn Request(&self, cx: *mut JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        // Step 3.
        let p = Promise::new(&self.global());

        // Step 1.
        let root_desc = match create_root_descriptor(cx, permissionDesc) {
            Ok(descriptor) => descriptor,
            Err(message) => {
                p.reject_error(cx, Error::Type(message));
                return p;
            },
        };

        // Step 2.
        match root_desc.name {
            // TODO: Add support for other cases too.
            PermissionName::Bluetooth => {
                let type_desc = match create_bluetooth_descriptor(cx, permissionDesc) {
                    Ok(descriptor) => descriptor,
                    Err(message) => {
                        p.reject_error(cx, Error::Type(message));
                        return p;
                    },
                };
                bluetooth_permission_request_call(&self.global(), type_desc, &p, cx);
            },
            _ => {
                // Step 4: TODO: Add an async implementation instead of sync_default_permission_request_call
                sync_default_permission_request_call(&self.global(), root_desc, &p, cx);
            },
        };

        return p;
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#dom-permissions-revoke
    unsafe fn Revoke(&self, cx: *mut JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        let p = Promise::new(&self.global());

        // Step 1.
        let root_desc = match create_root_descriptor(cx, permissionDesc) {
            Ok(descriptor) => descriptor,
            Err(message) => {
                p.reject_error(cx, Error::Type(message));
                return p;
            },
        };

        // Step 2.
        match root_desc.name {
            // TODO: Add support for not default cases
            _ => {
                // TODO: Clarify Step 3 - 4
                // NOTE(zakorgy): The steps are not clear about that we should call here a permission revoke method
                // but it is mentioned in `https://w3c.github.io/permissions/#permission-revocation-algorithm`:
                // "Run by Permissions' revoke() method"

                // Step 5.
                return self.Query(cx, permissionDesc);
            },
        }
    }
}
