/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FooBinding::FooBinding::{self, FooMethods};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use dom::bindings::js::Root;
use dom::bluetoothdevice::BluetoothDevice;

#[dom_struct]
pub struct Foo {
    reflector: Reflector,
}

impl Foo {
    pub fn new_inherited() -> Foo {
        Foo {
            reflector: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> Root<Foo> {
        reflect_dom_object(box Foo::new_inherited(),
                           global,
                           FooBinding::Wrap)
    }
}

impl FooMethods for Foo {
    fn Devices(&self) -> Vec<Root<HTMLElement>> {
        Vec::new()
    }
}