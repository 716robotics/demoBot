use crossbeam_channel::unbounded;
use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Address, Channel, Pca9685};
use std::{thread, time::Duration, vec, time::Instant};
use gilrs::{Gilrs, Button, Event, Axis};
fn main() {
    let (s, r) = unbounded();
    let mut gilrs = Gilrs::new().unwrap();
    let (_id,controller) = gilrs.gamepads().next().unwrap();
    let mut johnny5 = Robot {l_drive: Motor{channel:0, sender:s.clone()},r_drive: Motor {channel:1, sender:s.clone()},
        shoot_wheel: Motor{channel:2, sender:s.clone()}, feed_wheel: Motor{channel:3, sender:s.clone()},
        shoot_timer: Instant::now(), shooting: false};
    //spawn PWM handler
    thread::spawn(move || {
        let dev = I2cdev::new("/dev/i2c-1").unwrap();
        let address = Address::from(0x40);
        let mut pwm = Pca9685::new(dev, address).unwrap();
        // Frequency of 50Hz as per Spark documentation
        pwm.set_prescale(122).unwrap();
        pwm.enable().unwrap();
        loop {
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
    //arcade drive
    loop{
        if controller.is_connected(){
            johnny5.l_drive.set(controller.value(Axis::RightStickY)+controller.value(Axis::RightStickY));
            johnny5.r_drive.set(controller.value(Axis::RightStickY)-controller.value(Axis::RightStickY));
            johnny5.shoot(controller.value(Axis::RightZ));
        }
        else {
            johnny5.l_drive.set(0.0);
            johnny5.r_drive.set(0.0);
            johnny5.shoot_wheel.set(0.0);
            johnny5.feed_wheel.set(0.0);
            panic!("Controller disconnected!")
        }
    }

}
struct Motor{
    channel: u16,
    sender: crossbeam_channel::Sender<std::vec::Vec<u16>>
}
impl Motor{
    fn set(&self,mut speed:f32){
        if speed.abs() > 1.0 {speed = speed / speed.abs()}
        let output = (speed*0.5+0.5)*205.0+205.0; //transpose from -1:1 to 0:1 then into range 205:409 for PWM
        self.sender.send(vec!(self.channel,output.floor() as u16)).unwrap();
    }
}

struct Robot{
    l_drive: Motor,
    r_drive: Motor,
    shoot_wheel: Motor,
    feed_wheel: Motor,
    shoot_timer: Instant,
    shooting: bool
}
impl Robot{
    fn shoot(&mut self,trigger: f32){
        if trigger > 0.8{
            self.shooting = true;
            self.shoot_wheel.set(1.0);
            if self.shoot_timer.elapsed().as_millis() > 2000{
                //using millis so it triggers exactly after 2s
                self.feed_wheel.set(1.0);
            }
        }
        else {
            if self.shooting{
            self.feed_wheel.set(0.0);
            self.shoot_wheel.set(0.0);
            self.shooting = false;
            }
            self.shoot_timer = Instant::now();
        }
    }
}