mod motor;
mod profile;
use csv::Writer;
use motor::Motor;
use opencv::core::{bitwise_and, in_range, Mat, Point, Rect, Scalar, Vector, CV_8U};
use opencv::highgui::WND_PROP_AUTOSIZE;
use opencv::imgproc::{
    contour_area, draw_contours, find_contours, moments, CHAIN_APPROX_NONE, LINE_8, RETR_EXTERNAL,
};
use opencv::videoio::{VideoCaptureTrait, VideoCaptureTraitConst};
use opencv::{highgui, videoio, Result};
use profile::MotionProfile;
use rppal::pwm::{Channel, Polarity, Pwm};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Empty;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::{Duration, Instant};
use std::{error::Error, sync::mpsc};
use tract_core::internal::*;
use tract_ndarray::Array;
use tract_onnx::prelude::*;
#[derive(Debug)]
enum MotorMessage {
    SmoothTo(f64),
    HarshTo(f64),
    DutyTo(f64),
    PulseTo(u64),
    GetSpeed,
    Stop,
}

#[derive(Debug)]
enum MainMessage {
    Speed(f64),
}
#[derive(Debug)]
enum DaqMessage {
    Stop,
    Data(ControlLog),
}

#[derive(Debug, Serialize, Deserialize)]
struct ControlLog {
    angle_error: f64,
    offset_error: f64,
    speed: f64,
    angle: f64,
}
// 0.167 % every 1 ms
// 0.833 every tick at a 200Hz phased loop
fn main() -> Result<(), Box<dyn Error>> {
    let model = tract_onnx::onnx()
        // load the model
        .model_for_path("driver.onnx")?
        // specify input type and shape
        .with_input_fact(0, f32::fact([2]).into())?
        // optimize the model
        .into_optimized()?
        // make the model runnable and fix its inputs and outputs
        .into_runnable()?;
    let (motor_send, motor_recieve) = mpsc::channel();
    let (main_send, main_recieve) = mpsc::channel();
    let ctrlc_send = motor_send.clone();
    thread::spawn(move || {
        let mut profile: Option<MotionProfile> = None;
        let mut motor = Motor::new(Channel::Pwm1, (1300, 2000));
        println!("Motor warmup started");
        motor.warmup(1290);
        thread::sleep(Duration::from_millis(1000));
        println!("Motor Warmup done");
        let mut tick = 0;
        loop {
            if let Some(profil) = &mut profile {
                if profil.done(tick) {
                    profile = None;
                } else {
                    motor.set_speed(profil.probe(tick));
                }
            }
            match motor_recieve.try_recv() {
                Ok(value) => {
                    println!("{:?}", value);
                    match value {
                        MotorMessage::SmoothTo(num) => {
                            profile = Some(MotionProfile::new(motor.speed(), num, 0.16, tick));
                        }
                        MotorMessage::DutyTo(num) => {
                            motor.set_duty(num);
                        }
                        MotorMessage::HarshTo(num) => {
                            profile = None;
                            motor.set_speed(num);
                        }
                        MotorMessage::GetSpeed => {
                            main_send.send(MainMessage::Speed(motor.speed())).unwrap();
                        }
                        MotorMessage::PulseTo(num) => {
                            motor.set_pulse(Duration::from_micros(num));
                        }
                        MotorMessage::Stop => {
                            motor.disable();
                            break;
                        }
                    }
                }
                Err(error) => match error {
                    TryRecvError::Empty => {}
                    TryRecvError::Disconnected => panic!("Disconnected"),
                },
            }
            thread::sleep(Duration::from_millis(5));
            tick += 1;
        }
    });
    let (daq_send, daq_recieve) = mpsc::channel();
    let ctrlc_daq_send = daq_send.clone();
    thread::spawn(move || {
        let log_file = File::create(format!(
            "./logs/{}",
            chrono::prelude::Utc::now().to_string()
        ))
        .unwrap();
        let mut writer = Writer::from_writer(log_file);

        loop {
            match daq_recieve.recv().unwrap() {
                DaqMessage::Stop => {
                    break;
                }
                DaqMessage::Data(msg) => {
                    writer.serialize(msg).unwrap();
                }
            }
        }
    });
    thread::sleep(Duration::from_millis(2000));
    let servo_pwm = Pwm::with_frequency(Channel::Pwm0, 50.0, 0.077, Polarity::Normal, true)?;
    // servo_pwm.set_duty_cycle(0.09).unwrap();
    // println!("Motor should be moving");
    // motor_send.send(MotorMessage::HarshTo(0.175))?;
    let max_speed = 0.19;
    let min_speed = 0.175;
    ctrlc::set_handler(move || {
        ctrlc_daq_send.send(DaqMessage::Stop).unwrap();
        ctrlc_send.clone().send(MotorMessage::Stop).unwrap();

        Pwm::with_frequency(Channel::Pwm0, 50.0, 0.0, Polarity::Normal, false)
            .unwrap()
            .disable()
            .unwrap();
        thread::sleep(Duration::from_millis(500));
    })
    .unwrap();
    let masked_disp = "maskedecehce";
    highgui::named_window(masked_disp, WND_PROP_AUTOSIZE);
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera

    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    let bounds = 30;
    // RGB -> Name
    //144, 151, 157 -> white lane lines
    //155, 91, 47 -> orange lane lines
    let low = Vector::from_slice(&[47 - bounds, 91 - bounds, 155 - bounds]);
    let high = Vector::from_slice(&[47 + bounds, 91 + bounds, 155 + bounds]);
    loop {
        let t0 = Instant::now();
        let mut frame = Mat::default();
        cam.read(&mut frame)?;
        // highgui::imshow(masked_disp, &frame)?;
        // println!("{:?}", frame);
        let mut mask = Mat::default();
        in_range(&frame, &low, &high, &mut mask)?;
        let mut slices = vec![];
        for x in 0..5 {
            slices.push(Mat::roi(&mask.clone(), Rect::new(0, x * 50, 640, x * 50))?);
        }
        let mut points = vec![];
        for (idx, slice) in slices.iter().enumerate() {
            let mut contours: Vector<Vector<Point>> = Vector::new();

            find_contours(
                slice,
                &mut contours,
                RETR_EXTERNAL,
                CHAIN_APPROX_NONE,
                Point::default(),
            )?;
            if !contours.is_empty() {
                let mut biggest_mask = Mat::new_rows_cols_with_default(
                    50,
                    640,
                    CV_8U,
                    Scalar::new(0.0, 0.0, 0.0, 1.0),
                )?;
                let biggest_contour = contours
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, x)| (contour_area(x, false).unwrap() * 300.0) as i32)
                    .unwrap();
                draw_contours(
                    &mut biggest_mask,
                    &contours,
                    biggest_contour.0 as i32,
                    Scalar::new(255.0, 255.0, 255.0, 1.0),
                    -1,
                    LINE_8,
                    &Mat::default(),
                    100,
                    Point::default(),
                )?;

                let moments = moments(&biggest_mask, true)?;
                let center = Point::new(
                    (moments.m10 / moments.m00) as i32,
                    (moments.m01 / moments.m00) as i32,
                );
                // transpose the points to make the graph horizontal
                points.push((50 * (idx as i32) + center.y, center.x));
            }
        }

        if points.len() >= 3 {
            let point0 = points[0];
            let point1 = points[2];
            let p_steering_gain = -1.0;
            let offset_error = (320.0 - point0.1 as f64) / 320.0;
            let angle_error = ((point0.1 - point1.1) as f64 / 320.0).clamp(-1.0, 1.0);

            println!(
                "Offset error : {}, Angle Error: {}",
                offset_error, angle_error
            );
            // let effort = p_steering_gain * offset_error;
            // let zero = 0.077;
            // let turning_effort = zero + 0.02 * (effort).clamp(-1.0, 1.0);
            // let p_drive_gain = 0.3;
            // let speed = min_speed
            //     + (1.0 - (angle_error.abs()) + p_drive_gain)
            //         .powf(4.0)
            //         .clamp(0.0, 1.0)
            //         * (max_speed - min_speed);
            // // highgui::imshow(masked_disp, &sliced)?;

            let x: Tensor =
                tract_ndarray::Array1::from_vec(vec![offset_error as f32, angle_error as f32])
                    .into();
            let result = model.run(tvec!(x.into()))?;

            // find and display the max value with its index
            let best = result[0].to_array_view::<f32>()?;
            let turning_effort = best[0] as f64;
            let speed = best[1] as f64;
            let t1 = Instant::now();
            daq_send
                .send(DaqMessage::Data(ControlLog {
                    angle_error,
                    offset_error,
                    speed,
                    angle: turning_effort,
                }))
                .unwrap();
            println!(
                "Duty effort: {}, speed: {}, compute time: {:?}, points: {:?}",
                turning_effort,
                speed,
                t1 - t0,
                points
            );

            motor_send.send(MotorMessage::HarshTo(speed))?;
            servo_pwm.set_duty_cycle(turning_effort)?;
        }
        let key = highgui::wait_key(2)?;
        if key > 0 && key != 255 {
            break;
        }
    }
    Ok(())
}

// fn main() -> TractResult<()> {
//     let model = tract_onnx::onnx()
//         // load the model
//         .model_for_path("driver.onnx")?
//         // specify input type and shape
//         .with_input_fact(0, f32::fact([2]).into())?
//         // optimize the model
//         .into_optimized()?
//         // make the model runnable and fix its inputs and outputs
//         .into_runnable()?;
//     let x: Tensor = tract_ndarray::Array1::from_vec(vec![0.5_f32, 0.3]).into();
//     let result = model.run(tvec!(x.into()))?;

//     // find and display the max value with its index
//     let best = result[0].to_array_view::<f32>()?;
//     println!("result: {:?}, {:?}", best[0], best[1]);
//     Ok(())
// }
