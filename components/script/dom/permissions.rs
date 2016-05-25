/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding::BluetoothPermissionDescriptor;
use dom::bindings::codegen::Bindings::PermissionsBinding::{self, PermissionsMethods, PermissionName, PermissionNameValues};
use dom::bindings::codegen::Bindings::PermissionStatusBinding::{PermissionStorage, PermissionState};
//use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::Error::Type;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::permissionstatus::PermissionStatus;
use std::cell::RefCell;
use std::clone::Clone;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

thread_local!(pub static PERMISSION_STORE: RefCell<PermissionStore> = RefCell::new(PermissionStore(HashMap::new())));

pub type PermissionStorageID = (PermissionName, String/*, Option<String>*/);

impl Eq for PermissionName {}

impl Hash for PermissionName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        PermissionNameValues::strings[*self as usize].to_string().hash(state);
    }
}

impl Clone for PermissionStorage {
    fn clone(&self) -> PermissionStorage {
        PermissionStorage {
            state: self.state.clone(),
        }
    }
}

// https://w3c.github.io/permissions/#ref-for-permission-storage-identifier
fn get_permission_storage_id(name: &PermissionName, globalref: GlobalRef) -> PermissionStorageID {
    let mut document_origin = String::new();
    let mut top_level_origin = String::new();
    match globalref {
        GlobalRef::Window(window) => {
            document_origin = window.get_url().origin().ascii_serialization();
            // top_level_origin = window.Top().get_url().origin().ascii_serialization();
            (name.clone(), document_origin/*, Some(top_level_origin)*/)
        },
        GlobalRef::Worker(worker) => {
            document_origin = worker.get_url().origin().ascii_serialization();
            (name.clone(), document_origin/*, None*/)
        },
    }
}

pub struct PermissionStore(HashMap<PermissionStorageID,PermissionStorage>);

impl PermissionStore {
    // https://w3c.github.io/permissions/#retrieve-a-permission-storage-entry
    fn retrieve_permission_storage_entry(&self, ps_id: &PermissionStorageID) -> Option<&PermissionStorage> {
        return self.0.get(ps_id)
    }
    // https://w3c.github.io/permissions/#create-a-permission-storage-entry
    fn create_permission_storage_entry(&mut self, ps_id: &PermissionStorageID, pstorage: PermissionStorage) {
        self.0.insert(ps_id.clone(), pstorage);
    }
    // https://w3c.github.io/permissions/#delete-a-permission-storage-entry
    fn delete_permission_storage_entry(&mut self, ps_id: &PermissionStorageID) {
        self.0.remove(ps_id);
    }
    // https://w3c.github.io/permissions/#retrieve-the-permission-storage
    fn retrieve_permission_storage(&self, name: &PermissionName, globalref: GlobalRef) -> PermissionStorage {
        let id = get_permission_storage_id(name, globalref);
        match self.retrieve_permission_storage_entry(&id) {
            Some(pstorage) => pstorage.clone(),
            None => {
                return PermissionStorage { state: PermissionState::Denied, };
            },
        }
    }
}

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

    pub fn new(global: GlobalRef) -> Root<Permissions> {
        reflect_dom_object(box Permissions::new_inherited(),
                           global,
                           PermissionsBinding::Wrap)
    }
}

impl PermissionsMethods for Permissions {
    fn Query(&self, _permissionDesc: &PermissionDescriptor) -> Fallible<Root<PermissionStatus>> {
        Err(Type(String::from("")))
    }

    fn Request(&self, _permissionDesc: &PermissionDescriptor) -> Fallible<Root<PermissionStatus>> {
        Err(Type(String::from("")))
    }
}