extern crate ffmpeg_next as ffmpeg;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use std::fs::File;
use std::fs;
use std::io::Write;
use std::{thread, time::Duration};
use image::{GenericImageView};
use csv::Writer;
use std::error::Error;
use std::process::Command;


fn main() -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path("frames.csv")?;
    let v: Vec<String> =  do_stuff("rust.mp4".to_string());
    for i in &v {
        wtr.write_record(&[i, ""]);
    }
    Command::new("python3").arg("output.py").spawn();
    Ok(())
}

fn do_stuff(path: String) -> Vec<String> {
    ffmpeg::init().unwrap();
    let mut v: Vec<String> = Vec::new();
    if let Ok(mut ictx) = input(&format!("{}", path)) {
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound).expect("error");
        let video_stream_index = input.index();

        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters()).expect("error");
        let mut decoder = context_decoder.decoder().video().expect("error");

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        ).expect("error");

        let mut frame_index = 0;

        let mut receive_and_process_decoded_frames =
            |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;
                    save_file(&rgb_frame, frame_index, &mut v).unwrap();
                    frame_index += 1;
                }
                Ok(())
            };

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet).expect("error");
                receive_and_process_decoded_frames(&mut decoder).expect("error");
            }
        }
        decoder.send_eof().expect("error");
        receive_and_process_decoded_frames(&mut decoder).expect("error");
    }
    
    
    v
    
}
fn save_file(frame: &Video, index: usize, v: &mut Vec<String>) -> std::result::Result<(), std::io::Error> {
    let mut file = File::create(format!("frames/frame{}.ppm", index))?;
    file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    file.write_all(frame.data(0))?;
    v.push(get_image(&format!("frames/frame{}.ppm", index), 4));
    fs::remove_file(format!("frames/frame{}.ppm", index));
    Ok(())
}

fn get_str_ascii(intent: u8) -> &'static str {
    let index = intent/32;
    let ascii = [" ", ".", ",", "-", "~", "+", "=", "@"];
    return ascii[index as usize];
}

pub fn get_image(dir: &str, scale: u32) -> String {
    let mut output = String::new();
    let img = image::open(dir).unwrap();
    let (width, height) = img.dimensions();
    for y in 0..height {
        for x in 0..width {
            if y % (scale * 2) == 0 && x % scale == 0 {
                let pix = img.get_pixel(x, y);
                let mut intent = pix[0]/3 + pix[1]/3 + pix[2]/3;
                if pix[3] == 0 {
                    intent = 0;
                }
                output.push_str(&format!("{}", get_str_ascii(intent)));
            }
        }
        if y%(scale*2) == 0{
            output.push_str("\n");
        }
    }
    output
}