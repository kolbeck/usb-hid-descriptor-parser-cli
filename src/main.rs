use std::{io, process::exit};

use rusb::{GlobalContext, Device};

const HID_CLASS_CODE: u8 = 3;

fn main() {
    let devices: Vec<Device<GlobalContext>> = devices();
    if devices.is_empty() {
        println!("No USB HID devices found.");
        exit(0);
    }

    let mut i: usize = 0;
    for device in &devices {
        print_device(&device, i);
        i += 1;
    }

    println!("Select device to read HID descriptor from: ");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let device_index: usize = input.trim().parse().unwrap();
    let device = devices.get(device_index).unwrap();
    match usb_hid_descriptor_parser::get_hid_descriptor_bytes(device) {
        Ok(hid_bytes) => {
            print_hid_bytes(&hid_bytes);
            print_hid_report(&hid_bytes);
        },
        Err(error) => println!("Error printing USB config: {}", error),
    }
}

fn devices() -> Vec<Device<GlobalContext>> {
    let mut devices: Vec<Device<GlobalContext>> = vec![];
    for device in rusb::devices().unwrap().iter() {
        if is_hid_device(&device) {
            devices.push(device);
        }
    }

    devices
}

fn is_hid_device(device: &Device<GlobalContext>) -> bool {
    // TODO: Handle error
    let config = &device.config_descriptor(0).unwrap();
    for interface in config.interfaces() {
        for interface_desc in interface.descriptors() {
            if interface_desc.class_code() == HID_CLASS_CODE {
                return true;
            }
        }
    }

    false
}

fn print_device(device: &Device<GlobalContext>, index: usize) {
    let device_desc = match device.device_descriptor() {
        Ok(device_desc) => device_desc,
        Err(error) => {
            println!("Error: {}", error);
            return;
        },
    };
    print!(
        "{} {:04x}:{:04x}",
        index,
        device_desc.vendor_id(),
        device_desc.product_id(),
    );

    let device_handle = match device.open() {
        Ok(device_handle) => device_handle,
        Err(error) => {
            println!(" (Error: {})", error);
            return;
        }
    };
    let sn = match device_handle.read_serial_number_string_ascii(&device_desc) {
        Ok(sn) => sn,
        Err(error) => {
            println!(" (Error: {})", error);
            return;
        },
    };
    let manu = match device_handle.read_manufacturer_string_ascii(&device_desc) {
        Ok(manu) => manu,
        Err(error) => {
            println!(" (Error: {})", error);
            return;
        },
    };
    let name = match device_handle.read_product_string_ascii(&device_desc) {
        Ok(name) => name,
        Err(error) => {
            println!(" (Error: {})", error);
            return;
        },
    };

    println!(
        " [SN: {}] [Manu: {}] [Name: {}]",
        sn,
        manu,
        name,
    );
}

fn print_hid_bytes(hid_bytes: &Vec<u8>) {
    println!("\nUSB HID report descriptor bytes:");
    for byte in hid_bytes {
        print!("{:#04X} ", byte);
    }
    println!();
}

fn print_hid_report(hid_bytes: &Vec<u8>) {
    println!("\nUSB HID report descriptor:");
    let hid_report = usb_hid_descriptor_parser::hid::descriptor::get_descriptor_report(&hid_bytes);

    let mut longest_item_count: usize = 0;
    let mut collection_index: usize = 0;

    for item in &hid_report.items {
        if item.bytes.len() > longest_item_count {
            longest_item_count = item.bytes.len();
        }
    }
    
    for item in &hid_report.items {
        match &item.main_tag {
            Some(tag) => {
                match tag {
                    usb_hid_descriptor_parser::hid::descriptor::HidMainTag::EndCollection => collection_index -= 1,
                    _ => {}
                }
            },
            _ => {}
        }

        for byte in &item.bytes {
            print!("{:#04x}, ", byte);
        }
        println!(
            "{0:indent$}// {0:colindent$}{1}",
            "",
            item,
            indent = ((longest_item_count - item.bytes.len()) * 6) as usize,
            colindent = (collection_index * 2) as usize,
        );

        match &item.main_tag {
            Some(tag) => {
                match tag {
                    usb_hid_descriptor_parser::hid::descriptor::HidMainTag::Collection(_) => collection_index += 1,
                    _ => {}
                }
            },
            _ => {}
        }
    }
}
