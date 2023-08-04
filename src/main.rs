// // pwm_servo.rs - Rotates a servo using hardware PWM.
// //
// // Calibrate your servo beforehand, and change the values listed below to fall
// // within your servo's safe limits to prevent potential damage. Don't power the
// // servo directly from the Pi's GPIO header. Current spikes during power-up and
// // stalls could otherwise damage your Pi, or cause your Pi to spontaneously
// // reboot, corrupting your microSD card. If you're powering the servo using a
// // separate power supply, remember to connect the grounds of the Pi and the
// // power supply together.
// //
// // Interrupting the process by pressing Ctrl-C causes the application to exit
// // immediately without disabling the PWM channel. Check out the
// // gpio_blinkled_signals.rs example to learn how to properly handle incoming
// // signals to prevent an abnormal termination.
//
// mod motor;
// mod profile;
//
// use std::fs::File;
// use std::io::Empty;
// use std::sync::mpsc::TryRecvError;
// use std::thread;
// use std::time::Duration;
// use std::{error::Error, sync::mpsc};
//
// use motor::Motor;
// use profile::MotionProfile;
// use rascam::{info, SimpleCamera};
// use rppal::pwm::Channel;
//
// enum MotorMessage {
//     SmoothTo(f64),
//     HarshTo(f64),
//     GetSpeed,
// }
//
// #[derive(Debug)]
// enum MainMessage {
//     Speed(f64),
// }
//
// // 0.167 % every 1 ms
// // 0.833 every tick at a 200Hz phased loop
// fn main() -> Result<(), Box<dyn Error>> {
//     let (motor_send, motor_recieve) = mpsc::channel();
//     let (main_send, main_recieve) = mpsc::channel();
//     thread::spawn(move || {
//         let mut profile: Option<MotionProfile> = None;
//         let mut motor = Motor::new(Channel::Pwm0, (1200, 2000));
//         motor.warmup();
//         thread::sleep(Duration::from_millis(1000));
//         let mut tick = 0;
//         loop {
//             if let Some(profil) = &mut profile {
//                 if profil.done(tick) {
//                     profile = None;
//                 } else {
//                     motor.set_speed(profil.probe(tick));
//                 }
//             }
//             match motor_recieve.try_recv() {
//                 Ok(value) => match value {
//                     MotorMessage::SmoothTo(num) => {
//                         profile = Some(MotionProfile::new(motor.speed(), num, 0.16, tick));
//                     }
//                     MotorMessage::HarshTo(num) => {
//                         profile = None;
//                         motor.set_speed(num);
//                     }
//                     MotorMessage::GetSpeed => {
//                         main_send.send(MainMessage::Speed(motor.speed())).unwrap();
//                     }
//                 },
//                 Err(error) => match error {
//                     TryRecvError::Empty => {}
//                     TryRecvError::Disconnected => panic!("Disconnected"),
//                 },
//             }
//             thread::sleep(Duration::from_millis(5));
//             tick += 1;
//         }
//     });
//     motor_send.send(MotorMessage::SmoothTo(0.75)).unwrap();
//     for _ in 0..100 {
//         motor_send.send(MotorMessage::GetSpeed).unwrap();
//         let value = main_recieve.recv().unwrap();
//         println!("{:?}", value);
//     }
//     thread::sleep(Duration::from_millis(100));
//     motor_send.send(MotorMessage::HarshTo(0.0)).unwrap();
//     thread::sleep(Duration::from_millis(100));
//     Ok(())
//     // When the pwm variable goes out of scope, the PWM channel is automatically disabled.
//     // You can manually disable the channel by calling the Pwm::disable() method.
// }
use camera_capture;
use image;

use std::fs::File;
use std::path::Path;

fn main() {
    let cam = camera_capture::create(0).unwrap();

    let mut cam_iter = cam.fps(5.0).unwrap().start().unwrap();
    let img = cam_iter.next().unwrap();

    let file_name = "test.png";
    let path = Path::new(&file_name);
    let _ = &mut File::create(&path).unwrap();
    img.save(&path).unwrap();

    println!("img saved to {}", file_name);
}
