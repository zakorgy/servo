/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
 
dictionary PermissionDescriptor {
    required PermissionName name;
};

enum PermissionName {
    "geolocation",
    "notifications",
    "push",
    "midi",
    "camera",
    "microphone",
    "speaker",
    "device-info",
    "background-sync",
    "bluetooth",
};

dictionary PushPermissionDescriptor : PermissionDescriptor {
    boolean userVisibleOnly = false;
};

dictionary MidiPermissionDescriptor : PermissionDescriptor {
    boolean sysex = false;
};

dictionary DevicePermissionDescriptor : PermissionDescriptor {
    DOMString deviceId;
};

//[Exposed=(Window,Worker)]
interface Permissions {
    [Throws]
    PermissionStatus query(PermissionDescriptor permissionDesc);
    [Throws]
    PermissionStatus request(PermissionDescriptor permissionDesc);

    /*Promise<PermissionStatus> query(PermissionDescriptor permissionDesc);

    Promise<PermissionStatus> request(PermissionDescriptor permissionDesc);

    Promise<PermissionStatus> revoke(PermissionDescriptor permissionDesc);*/
};