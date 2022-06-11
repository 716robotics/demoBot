use crossbeam_channel::bounded;
use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Address, Channel, Pca9685};
use std::{thread, time::Duration, vec, num};
fn main() {
    let (s, r) = bounded(5);
    // define motors
    let lDrive = Motor {channel:0, sender:s.clone()};
    //spawn PWM handler
    thread::spawn(move || {
        let dev = I2cdev::new("/dev/i2c-1").unwrap();
        let address = Address::from(0x40);
        let mut pwm = Pca9685::new(dev, address).unwrap();
        // Frequency of 50Hz as per Spark documentation
        loop {
            pwm.set_prescale(122).unwrap();
            pwm.enable().unwrap();
            pwm.set_channel_on_off(Channel::C15, 0,409).unwrap();
            thread::sleep(Duration::from_millis(3000));
            
            for command in r.try_iter(){
                pwm.set_channel_on_off(int_to_channel(command[0]),0,command[1]).unwrap();
            }

        }
        fn int_to_channel(input: u16)->Channel{
            match input{
                0=>return Channel::C0,
                1=>return Channel::C1,
                2=>return Channel::C2,
                3=>return Channel::C3,
                4=>return Channel::C4,
                5=>return Channel::C5,
                6=>return Channel::C6,
                7=>return Channel::C7,
                8=>return Channel::C8,
                9=>return Channel::C9,
                10=>return Channel::C10,
                11=>return Channel::C11,
                12=>return Channel::C12,
                13=>return Channel::C13,
                14=>return Channel::C14,
                _=>return Channel::C15
            }
        }
    });

}
struct Motor{
    channel: u16,
    sender: crossbeam_channel::Sender<std::vec::Vec<u16>>
}
impl Motor{
    fn set(&self, speed:f32){
        let output = (speed*0.5+0.5)*205.0+205.0; //transpose from -1:1 to 0:1 then into range 205:409 for PWM
        self.sender.send(vec!(self.channel,output.floor() as u16)).unwrap();
    }
}