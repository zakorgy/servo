function clear() {
    document.getElementById("log").textContent = "";
}

function log(line) {
    document.getElementById("log").textContent += line + '\n';
}

function AsciiToDecimal(bytestr) {
    var result = [];
    for(i = 0; i < bytestr.length; i++) {
        result[i] = bytestr[i].charCodeAt(0) ;
    }
    return result;
}

function populate(testCases){
	for(i = 0; i < testCases.length; ++i) {
        var btn = document.createElement('button');
        btn.setAttribute('onclick','onButtonClick(' + i + ')');
        btn.innerHTML = 'Test '+ (i+1);
        document.getElementById('buttons').appendChild(btn);
    }
}

var characteristicReadValueTestCases = [];
//Test 1
    characteristicReadValueTestCases.push({characteristic: 'body_sensor_location', mustDisconnect: true});
//Test 2
    characteristicReadValueTestCases.push({characteristic: 'gap.reconnection_address', mustDisconnect: false});
//Test 3
    characteristicReadValueTestCases.push({characteristic: 'serial_number_string', mustDisconnect: false});
//Test 4
    characteristicReadValueTestCases.push({characteristic: 0x00002a03, mustDisconnect: false});
//Test 5
    characteristicReadValueTestCases.push({characteristic: 0x00002a25, mustDisconnect: false});
//Test 6
    characteristicReadValueTestCases.push({characteristic: '00002a03-0000-1000-8000-00805f9b34fb', mustDisconnect: false});
//Test 7
    characteristicReadValueTestCases.push({characteristic: '00002a25-0000-1000-8000-00805f9b34fb', mustDisconnect: false});
//Test 8
    characteristicReadValueTestCases.push({characteristic: 'body_sensor_location', mustDisconnect: false});
//Test 9
    characteristicReadValueTestCases.push({characteristic: 0x00002a38, mustDisconnect: false});
//Test 10
    characteristicReadValueTestCases.push({characteristic: '00002a38-0000-1000-8000-00805f9b34fb', mustDisconnect: false});
//Test 11
    characteristicReadValueTestCases.push({characteristic: 'heart_rate_control_point', mustDisconnect: false});

var characteristicWriteValueTestCases = [];
//Test 1
    characteristicWriteValueTestCases.push({characteristic: 0x2345, valueToWrite: [11], mustDisconnect: true});
//Test 2
    characteristicWriteValueTestCases.push({characteristic: 0x2345, valueToWrite: new Array(513), mustDisconnect: false});
//Test 3
    characteristicWriteValueTestCases.push({characteristic: 'gap.reconnection_address', valueToWrite: [1], mustDisconnect: false});
//Test 4
    characteristicWriteValueTestCases.push({characteristic: 'serial_number_string', valueToWrite: [2], mustDisconnect: false});
//Test 5
    characteristicWriteValueTestCases.push({characteristic: 0x00002a02, valueToWrite: [3], mustDisconnect: false});
//Test 6
    characteristicWriteValueTestCases.push({characteristic: 0x00002a03, valueToWrite: [3], mustDisconnect: false});
//Test 7
    characteristicWriteValueTestCases.push({characteristic: 0x00002a25, valueToWrite: [4], mustDisconnect: false});
//Test 8
    characteristicWriteValueTestCases.push({characteristic: '00002a02-0000-1000-8000-00805f9b34fb', valueToWrite: [6], mustDisconnect: false});
//Test 9
    characteristicWriteValueTestCases.push({characteristic: '00002a03-0000-1000-8000-00805f9b34fb', valueToWrite: [5], mustDisconnect: false});
//Test 10
    characteristicWriteValueTestCases.push({characteristic: '00002a25-0000-1000-8000-00805f9b34fb', valueToWrite: [6], mustDisconnect: false});
//Test 11
    characteristicWriteValueTestCases.push({characteristic: 0x2345, valueToWrite: [11]});
//Test 12
    characteristicWriteValueTestCases.push({characteristic: '00002345-0000-1000-8000-00805f9b34fb', valueToWrite: [22], mustDisconnect: false});

var getCharacteristicTestCases = [];
//Test 1
    getCharacteristicTestCases.push({characteristic: 'not_a_characteristic_name', service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 2
    getCharacteristicTestCases.push({characteristic: 'battery_level', service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 3
    getCharacteristicTestCases.push({characteristic: '1234567891000-1000-8000-00805f9b34fb', service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 4
    getCharacteristicTestCases.push({characteristic: '11', service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 5
    getCharacteristicTestCases.push({characteristic: '12345678-1234-1234-1234-123456789abc', service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 6
    getCharacteristicTestCases.push({characteristic: '00000000-0000-0000-0000-000000000000', service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 7
    getCharacteristicTestCases.push({characteristic: 0x0000, service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 8
    getCharacteristicTestCases.push({characteristic: 0x000000000, service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 9
    getCharacteristicTestCases.push({characteristic: 0x2a19, service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 10
    getCharacteristicTestCases.push({characteristic: 0x12345678, service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 11
    getCharacteristicTestCases.push({characteristic: 0x00002a19, service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 12
    getCharacteristicTestCases.push({characteristic: 0x00002a03, service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 13
    getCharacteristicTestCases.push({characteristic: 0x00002a25, service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 14
    getCharacteristicTestCases.push({characteristic: 0x2a03, service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 15
    getCharacteristicTestCases.push({characteristic: 0x2a25, service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 16
    getCharacteristicTestCases.push({characteristic: '00002a03-0000-1000-8000-00805f9b34fb', service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 17
    getCharacteristicTestCases.push({characteristic: '00002a25-0000-1000-8000-00805f9b34fb', service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });

var getCharacteristicsTestCases = [];
//Test 1
    getCharacteristicsTestCases.push({service: 'battery_service', options: {filters: [{services: ['battery_service']}], optionalServices: ['cycling_power']} });
//Test 2
    getCharacteristicsTestCases.push({characteristic: 'not_a_characteristic_name', service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 3
    getCharacteristicsTestCases.push({characteristic: 'body_sensor_location', service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 4
    getCharacteristicsTestCases.push({characteristic: '1234567891000-1000-8000-00805f9b34fb', service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 5
    getCharacteristicsTestCases.push({characteristic: '11', service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 6
    getCharacteristicsTestCases.push({characteristic: '12345678-1234-1234-1234-123456789abc', service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 7
    getCharacteristicsTestCases.push({characteristic: '00000000-0000-0000-0000-000000000000', service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 8
    getCharacteristicsTestCases.push({characteristic: 0x0000, service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 9
    getCharacteristicsTestCases.push({characteristic: 0x000000000, service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 10
    getCharacteristicsTestCases.push({characteristic: 0x2a38, service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 11
    getCharacteristicsTestCases.push({characteristic: 0x12345678, service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 12
    getCharacteristicsTestCases.push({characteristic: 0x00002a38, service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 13
    getCharacteristicsTestCases.push({characteristic: 0x00002a03, service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 14
    getCharacteristicsTestCases.push({characteristic: 0x00002a25, service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 15
    getCharacteristicsTestCases.push({characteristic: 0x2a03, service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 16
    getCharacteristicsTestCases.push({characteristic: 0x2a25, service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 17
    getCharacteristicsTestCases.push({characteristic: '00002a03-0000-1000-8000-00805f9b34fb', service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });
//Test 18
    getCharacteristicsTestCases.push({characteristic: '00002a25-0000-1000-8000-00805f9b34fb', service: 'heart_rate', options: {filters: [{services: ['heart_rate']}], optionalServices: ['cycling_power']} });

var descriptorReadValueTestCases = [];
//Test 1
    descriptorReadValueTestCases.push({descriptor: 'gatt.client_characteristic_configuration', mustDisconnect: true});
//Test 2
    descriptorReadValueTestCases.push({descriptor: 0x2902, mustDisconnect: true});
//Test 3
    descriptorReadValueTestCases.push({descriptor: '00002902-0000-1000-8000-00805f9b34fb', mustDisconnect: true});
//Test 4
    descriptorReadValueTestCases.push({descriptor: 'gatt.client_characteristic_configuration', mustDisconnect: false});
//Test 5
    descriptorReadValueTestCases.push({descriptor: 0x2902, mustDisconnect: false});
//Test 6
    descriptorReadValueTestCases.push({descriptor: '00002902-0000-1000-8000-00805f9b34fb', mustDisconnect: false});

var descriptorWriteValueTestCases = [];
//Test 1
    descriptorWriteValueTestCases.push({descriptor: '00003456-0000-1000-8000-00805f9b34fb', valueToWrite: [11], mustDisconnect: true});
//Test 2
    descriptorWriteValueTestCases.push({descriptor: '00003456-0000-1000-8000-00805f9b34fb', valueToWrite: new Array(513), mustDisconnect: false});
//Test 3
    descriptorWriteValueTestCases.push({descriptor: '00002902-0000-1000-8000-00805f9b34fb', valueToWrite: [1], mustDisconnect: false});
//Test 4
    descriptorWriteValueTestCases.push({descriptor: 0x00002902, valueToWrite: [2], mustDisconnect: false});
//Test 5
    descriptorWriteValueTestCases.push({descriptor: 0x3456, valueToWrite: [11], mustDisconnect: false});
//Test 6
    descriptorWriteValueTestCases.push({descriptor: '00003456-0000-1000-8000-00805f9b34fb', valueToWrite: [22], mustDisconnect: false});
