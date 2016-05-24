/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

dictionary PermissionStorage {
    // PermissionStorage is just an explanatory device.
    // Instances are never received from or passed to Javascript code.

    required PermissionState state;
};

enum PermissionState {
    "granted",
    "denied",
    "prompt",
};

//[Exposed=(Window,Worker)]
interface PermissionStatus : EventTarget {
    readonly attribute PermissionState state;
    attribute EventHandler onchange;
};