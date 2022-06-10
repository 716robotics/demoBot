use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Address, Channel, Pca9685};
use std::{thread, time::Duration};
fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = Address::from(0x40);
    let mut pwm = Pca9685::new(dev, address).unwrap();
    // This corresponds to a frequency of 60 Hz.
    pwm.set_prescale(122).unwrap();

    // It is necessary to enable the device.
    pwm.enable().unwrap();
    pwm.set_channel_on_off(Channel::C15, 0,409).unwrap();
    thread::sleep(Duration::from_millis(3000));
    pwm.set_channel_on_off(Channel::C15, 0,307).unwrap();
    thread::sleep(Duration::from_millis(3000));
    pwm.set_channel_on_off(Channel::C15, 0,205).unwrap();
    thread::sleep(Duration::from_millis(3000));
    pwm.set_channel_on_off(Channel::C15, 0,307).unwrap();
    thread::sleep(Duration::from_millis(3000));
    pwm.set_channel_on_off(Channel::C15, 0,409).unwrap();
    thread::sleep(Duration::from_millis(3000));
    pwm.set_channel_on_off(Channel::C15, 0,307).unwrap();

    let _dev = pwm.destroy(); // Get the I2C device back
}