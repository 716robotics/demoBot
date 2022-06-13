use crossbeam_channel::bounded;
use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Address, Channel, Pca9685};
use std::{thread, vec, time::Instant};
use gilrs::{Gilrs, Button, Axis, EventType};
fn main() {
    let (s, r) = bounded(50);
    let (wds, wdr):(crossbeam_channel::Sender<Instant>,crossbeam_channel::Receiver<Instant>) = bounded(10);
    let mut gilrs = Gilrs::new().unwrap();
    let mut johnny5 = Robot {l_drive: Motor{channel:0, sender:s.clone()},r_drive: Motor {channel:1, sender:s.clone()},
        shoot_wheel: Motor{channel:2, sender:s.clone()}, feed_wheel: Motor{channel:3, sender:s.clone()},
        shoot_timer: Instant::now(), shooting: true, thr:vec!(0.0,0.0)};
    //spawn PWM handler
    thread::spawn(move || {
        let dev = I2cdev::new("/dev/i2c-1").unwrap();
        let address = Address::from(0x40);
        let mut pwm = Pca9685::new(dev, address).unwrap();
        // Frequency of 50Hz as per Spark documentation
        pwm.set_prescale(122).unwrap();
        pwm.enable().unwrap();
        let mut watchdog = Instant::now();
        let mut estop = false;
        loop {
            if r.len() > 35{
                //println!("PWM falling behind, Purging...");
                let _a:Vec<_>=r.try_iter().collect();
                drop(_a);
            }

            match r.try_iter().next(){
                Some(command) => {
                    if command[0] == 65535 {estop = command[1] == 0;}
                    if estop {pwm.set_channel_full_off(Channel::All).unwrap();
                    println!("Controller disconnected, E-stopped.")}
                    else {pwm.set_channel_on_off(int_to_channel(command[0]),0,command[1]).unwrap()}
                },
                None => ()
            }
            match wdr.try_iter().last(){
                Some(wd) => watchdog = wd,
                None => ()
            }
            if watchdog.elapsed().as_millis()>2500{
                pwm.set_channel_full_off(Channel::C0).unwrap();
                pwm.set_channel_full_off(Channel::C1).unwrap();
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
    let mut trigger = 0.0;
    loop {
        while let Some(ev) = gilrs.next_event() {
            wds.send(Instant::now()).unwrap();
            match ev.event {
                EventType::AxisChanged(axs,val,_) => {
                    match axs{
                        Axis::LeftStickY|Axis::LeftStickX => johnny5.drive(axs,val),
                        _ => (),
                    }
                }
                EventType::ButtonChanged(bttn,v,_) => {
                    match bttn{
                        Button::South =>trigger = v,
                        Button::East => s.send(vec!(65535,0)).unwrap(),
                        Button::North => s.send(vec!(65535,1)).unwrap(),
                        _ => ()
                    }
                }
                EventType::Disconnected => {println!("-------------\nDetected controller disconnect!!");
                    s.send(vec!(65535,0)).unwrap()},
                EventType::Connected => s.send(vec!(65535,1)).unwrap(),
                _ => (),
            }
        }
        johnny5.shoot(trigger);
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
        let tmp = vec!(self.channel,output.floor() as u16);
        self.sender.send(tmp).unwrap();
    }
}

struct Robot{
    l_drive: Motor,
    r_drive: Motor,
    shoot_wheel: Motor,
    feed_wheel: Motor,
    shoot_timer: Instant,
    shooting: bool,
    thr: Vec<f32>
}
impl Robot{
    fn shoot(&mut self,trigger: f32){
        if trigger >= 0.8{
            if !self.shooting{
                self.shooting = true;
                self.shoot_wheel.set(-1.0);
                self.shoot_timer = Instant::now();
            }
            if self.shoot_timer.elapsed().as_millis() > 2000{
                self.feed_wheel.set(-1.0);
                self.shoot_wheel.set(-1.0);
            }
        }
        else {
            if self.shooting{
                self.feed_wheel.set(0.0);
                self.shoot_wheel.set(0.0);
            }
            self.shooting = false;
        }
    }
    fn drive(&mut self,axis: gilrs::Axis, value: f32){
        match axis{
            Axis::LeftStickX => self.thr[0] = value,
            Axis::LeftStickY => self.thr[1] = value,
            _ => ()
        }
        self.l_drive.set(self.thr[1]+self.thr[0]);
        self.r_drive.set(self.thr[1]-self.thr[0]);
    }
}