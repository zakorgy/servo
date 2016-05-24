/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

dictionary BluetoothPermissionDescriptor : PermissionDescriptor {
    DOMString deviceId;
    // These match RequestDeviceOptions.
    // sequence<BluetoothRequestDeviceFilter> filters;
    sequence<BluetoothScanFilter> filters;
    sequence<BluetoothServiceUUID> optionalServices /*= []*/;
};

dictionary AllowedBluetoothDevice {
    required DOMString deviceId;
    required sequence<UUID> allowedServices;
};
dictionary BluetoothPermissionStorage : PermissionStorage {
    required sequence<AllowedBluetoothDevice> allowedDevices /*= []*/;
};

interface BluetoothPermissionResult : PermissionStatus {
    // attribute FrozenArray<BluetoothDevice> devices;
    // sequence<BluetoothDevice> getDevices(); maybe use this?
};