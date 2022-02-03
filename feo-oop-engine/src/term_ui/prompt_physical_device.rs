//! Constructs used for prompting for a physical device.
//! 
//! TODO
//! 
use std::sync::Arc;

use vulkano::instance::{
    PhysicalDevice,
    PhysicalDeviceType,
    Instance
};
use colored::Colorize;


pub fn prompt_physical_device(instance: &Arc<Instance>, index: Option<usize>) -> PhysicalDevice{
    let physical_devices = PhysicalDevice::enumerate(instance);
    match index {
        Some(index) if index < physical_devices.len() => {
            physical_devices.clone().nth(index).to_owned().unwrap()
        },
        _ => {
            println!("\tType\t\tName");

            physical_devices.clone().for_each(|device | {
                println!("{}\t{}\t{}", device.index(), match device.ty(){
                    PhysicalDeviceType::DiscreteGpu => "Discrete Gpu",
                    PhysicalDeviceType::IntegratedGpu => "Integrated Gpu",
                    PhysicalDeviceType::VirtualGpu => "Virtual Gpu",
                    PhysicalDeviceType::Other => "Other ",
                    PhysicalDeviceType::Cpu => "Cpu ",
                }, device.name());
            });
            
            println!("{}","Enter your preferred device :".blue().bold());
            let index = loop {
                let mut input = String::new();
		let s =  std::io::stdin().read_line(&mut input);
                if s.is_ok() {
                    match input.lines().next().unwrap().parse::<usize>() {
                        Ok(i) if i < physical_devices.clone().len() => {
                            break i;
                        },
                        _ => {}
                    }
                }
                println!("{}", "Please input a valid device.".red().bold());
            };

            physical_devices.clone().nth(index).to_owned().unwrap()
        }
    }
}
