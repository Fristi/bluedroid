use std::sync::{Arc, Mutex};

use crate::{
    gatt_server::Descriptor,
    utilities::{AttributePermissions, BleUuid},
};

use embedded_svc::storage::RawStorage;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::nvs_storage::EspNvsStorage;
use lazy_static::lazy_static;

lazy_static! {
    static ref STORAGE: Mutex<EspNvsStorage> = Mutex::new(
        EspNvsStorage::new_default(Arc::new(EspDefaultNvs::new().unwrap()), "ble", true).unwrap()
    );
}

impl Descriptor {
    pub fn user_description<S: AsRef<str>>(description: S) -> Self {
        Descriptor::new(
            "User Description",
            BleUuid::from_uuid16(0x2901),
            AttributePermissions::read(),
        )
        .set_value(description.as_ref().as_bytes().to_vec())
        .to_owned()
    }

    pub fn cccd() -> Self {
        Descriptor::new(
            "Client Characteristic Configuration",
            BleUuid::from_uuid16(0x2902),
            AttributePermissions::read_write(),
        )
        .on_read(|param| {
            let storage = STORAGE.lock().unwrap();

            // Create a key from the connection address.
            let key = format!(
                "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                param.bda[0], param.bda[1], param.bda[2], param.bda[3], param.bda[4], param.bda[5]
            );

            // Prepare buffer and read correct CCCD value from non-volatile storage.
            let mut buf: [u8; 2] = [0; 2];
            if let Some(value) = storage.get_raw(&key, &mut buf).unwrap() {
                value.0.to_vec()
            } else {
                vec![0, 0]
            }
        })
        .on_write(|value, param| {
            let mut storage = STORAGE.lock().unwrap();

            // Create a key from the connection address.
            let key = format!(
                "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                param.bda[0], param.bda[1], param.bda[2], param.bda[3], param.bda[4], param.bda[5]
            );

            // Write CCCD value to non-volatile storage.
            storage.put_raw(&key, &value).unwrap();
        })
        .to_owned()
    }
}
