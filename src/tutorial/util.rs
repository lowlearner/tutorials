use rusty_ffmpeg::ffi;
use std:: {
    fs::File,
    io::Write,
    slice,};

pub fn decode_packet(
    packet: &ffi::AVPacket,
    codec_context: &mut ffi::AVCodecContext,
    frame: &mut ffi::AVFrame,
) -> Result<(), String> {
    let mut response = unsafe { ffi::avcodec_send_packet(codec_context, packet) };

    if response < 0 {
        return Err(String::from("Error while sending a packet to the decoder."));
    }

    while response >= 0 {
        response = unsafe { ffi::avcodec_receive_frame(codec_context, frame) };
        if response == ffi::AVERROR(ffi::EAGAIN) || response == ffi::AVERROR_EOF {
            break;
        } else if response < 0 {
            return Err(String::from(
                "Error while receiving a frame from the decoder.",
            ));
        } else {
            println!(
                "Frame {} (type={}, size={} bytes) pts {} key_frame {} [DTS {}]",
                codec_context.frame_number,
                unsafe { ffi::av_get_picture_type_char(frame.pict_type) },
                frame.pkt_size,
                frame.pts,
                frame.key_frame,
                frame.coded_picture_number
            );

            let frame_filename = format!(
                "./media/output/pic/frame-{}.pgm",
                codec_context.frame_number
            );
            let width = frame.width as usize;
            let height = frame.height as usize;
            let wrap = frame.linesize[0] as usize;
            let data = unsafe { slice::from_raw_parts(frame.data[0], wrap * height) };
            save_gray_frame(data, wrap, width, height, frame_filename).unwrap();
        }
    }
    Ok(())
}



pub fn save_gray_frame(
    buf: &[u8],
    wrap: usize,
    xsize: usize,
    ysize: usize,
    filename: String,
) -> Result<(), std::io::Error> {
    let mut file = File::create(filename)?;
    let data = format!("P5\n{} {}\n{}\n", xsize, ysize, 255);
    file.write_all(data.as_bytes())?;

    for i in 0..ysize {
        file.write_all(&buf[i * wrap..(i * wrap + xsize)])?;
    }
    Ok(())
}