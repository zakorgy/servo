/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothRequest, GATTLevel};
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root, MutHeap, MutNullableHeap};
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::Bluetooth;
use dom::bluetoothadvertisingdata::BluetoothAdvertisingData;
use dom::bluetoothremotegattserver::BluetoothRemoteGATTServer;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use ipc_channel::ipc::{self, IpcSender};

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdevice
#[dom_struct]
pub struct BluetoothDevice {
    eventtarget: EventTarget,
    id: DOMString,
    name: Option<DOMString>,
    ad_data: MutHeap<JS<BluetoothAdvertisingData>>,
    gatt: MutNullableHeap<JS<BluetoothRemoteGATTServer>>,
    context: MutHeap<JS<Bluetooth>>,
}

impl BluetoothDevice {
    pub fn new_inherited(id: DOMString,
                         name: Option<DOMString>,
                         ad_data: &BluetoothAdvertisingData,
                         context: &Bluetooth)
                         -> BluetoothDevice {
        BluetoothDevice {
            eventtarget: EventTarget::new_inherited(),
            id: id,
            name: name,
            ad_data: MutHeap::new(ad_data),
            gatt: Default::default(),
            context: MutHeap::new(context),
        }
    }

    pub fn new(global: &GlobalScope,
               id: DOMString,
               name: Option<DOMString>,
               adData: &BluetoothAdvertisingData,
               context: &Bluetooth)
               -> Root<BluetoothDevice> {
        reflect_dom_object(box BluetoothDevice::new_inherited(id,
                                                              name,
                                                              adData,
                                                              context),
                           global,
                           BluetoothDeviceBinding::Wrap)
    }

    pub fn get_context(&self) -> Root<Bluetooth> {
        self.context.get()
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }

    pub fn is_represented_device_null(&self) -> bool {
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothRequest::IsRepresentedNull(self.Id().to_string(), GATTLevel::Device, sender)).unwrap();
        let result = receiver.recv().unwrap();
        result.unwrap_or(true)
    }

    pub fn set_represented_device_to_null(&self) {
        self.get_bluetooth_thread().send(
            BluetoothRequest::SetRepresentedToNull(self.Id().to_string(), GATTLevel::Device)).unwrap()
    }

    pub fn get_instance_ids_from_realm(&self) -> (Vec<String>, Vec<String>, Vec<String>) {
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetInstanceIds(self.Id().to_string(), sender)).unwrap();
        let result = receiver.recv().unwrap();
        result.unwrap_or((Vec::new(), Vec::new(), Vec::new()))
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#clean-up-the-disconnected-device
    #[allow(unrooted_must_root)]
    pub fn clean_up_disconnected_device(&self) {
        // Step 1.
        self.Gatt().set_connected(false);

        // TODO: Step 2: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

        // Step 3.
        // Note: Try without this variable binding!!!
        let context = self.get_context();

        let (service_ids, characteristic_ids, descriptor_ids) = self.get_instance_ids_from_realm();

        // Step 4, 5.
        let mut service_map = context.get_service_map().borrow_mut();
        for id in service_ids {
            if let Some(service) = service_map.remove(&id) {
                service.get().set_represented_service_to_null();
            }
        }

        // Step 4, 6.
        // TODO: Implement `active notification context set` for BluetoothRemoteGATTCharacteristic.
        let mut characteristic_map = context.get_characteristic_map().borrow_mut();
        for id in characteristic_ids {
            if let Some(characteristic) = characteristic_map.remove(&id) {
                characteristic.get().set_represented_characteristic_to_null();
            }
        }

        // Step 4, 7.
        let mut descriptor_map = context.get_descriptor_map().borrow_mut();
        for id in descriptor_ids {
            if let Some(descriptor) = descriptor_map.remove(&id) {
                descriptor.get().set_represented_descriptor_to_null();
            }
        }

        // Step 8.
        self.upcast::<EventTarget>().fire_bubbling_event(atom!("gattserverdisconnected"));

    }
}

impl BluetoothDeviceMethods for BluetoothDevice {
     // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-id
    fn Id(&self) -> DOMString {
        self.id.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-name
    fn GetName(&self) -> Option<DOMString> {
        self.name.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-addata
    fn AdData(&self) -> Root<BluetoothAdvertisingData> {
        self.ad_data.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-gatt
    fn Gatt(&self) -> Root<BluetoothRemoteGATTServer> {
        // TODO: Step 1 - 2: Implement the Permission API.
        self.gatt.or_init(|| {
            BluetoothRemoteGATTServer::new(&self.global(), self)
        })
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdeviceeventhandlers-ongattserverdisconnected
    event_handler!(gattserverdisconnected, GetOngattserverdisconnected, SetOngattserverdisconnected);
}
