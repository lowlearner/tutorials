use rusty_ffmpeg::ffi;

use std:: {ffi::{CString, CStr},
    ptr, slice,};
use super::util::decode_packet;

// 
pub fn save_frames() {
    //将所有可用的文件格式和编解码器注册到
    let filepath: CString = CString::new("./media/video/bear.mp4").unwrap();

    println!("initializing all the containers, codecs and protocols.");

    let mut format_context_ptr = unsafe { ffi::avformat_alloc_context() };

    if format_context_ptr.is_null() {
        panic!("ERROR could not allocate memory for FormatContext");
    }
    println!(
        "opening the input file ({}) and loading format (container) header",
        filepath.to_str().unwrap()
    );
    // 打开文件,此函数读取文件头，
    // 并将有关文件格式的信息存储在我们给定的 [AVFormatContext]结构中
    if unsafe {
        ffi::avformat_open_input(&mut format_context_ptr, filepath.as_ptr(), ptr::null_mut(), ptr::null_mut())
    } != 0 {
        panic!("ERROR could not open the file");
    }
    // 检出文件中的流信息。
    let format_context = unsafe { format_context_ptr.as_mut()}.unwrap();
    //读一些文件属性
    let format_name = unsafe {
        CStr::from_ptr((*format_context.iformat).name)
    }.to_str().unwrap();

    println!("format {}, duration {}, bitrate {}", format_name, format_context.duration, format_context.bit_rate);
    
    // 流信息填充到 `format_context->streams`中`
    if unsafe {
        ffi::avformat_find_stream_info(format_context, ptr::null_mut())
    } < 0 {
        panic!("ERROR couldn't find stream information");
    }

    //寻找视频流
    //定义解码器，解码上下文，流下标
    let mut codec_ptr : *const ffi::AVCodec = ptr::null_mut(); //初始化为null
    let mut codec_parament_ptr : *const ffi::AVCodecParameters = ptr::null_mut();
    let mut video_stream_index = None;
    
    let streams = unsafe{
        slice::from_raw_parts(format_context.streams, format_context.nb_streams as usize)
    };
    //遍历stream，保存video的codec和par
    for (i, stream) in streams
        .iter()
        .map(|stream| unsafe { stream.as_ref().unwrap()})
        .enumerate() {
            println!(
                "AVStream->time_base before open coded {}/{}",
                stream.time_base.num, stream.time_base.den
            );
            println!(
                "AVStream->r_frame_rate before open coded {}/{}",
                stream.r_frame_rate.num, stream.r_frame_rate.den
            );
            println!("AVStream->start_time {}", stream.start_time);
            println!("AVStream->duration {}", stream.duration);
            println!("finding the proper decoder (CODEC)");
            //使用解码器打开视频流
            let local_codec_par = unsafe { stream.codecpar.as_ref()}.unwrap() ;
            let local_codec = unsafe {
                ffi::avcodec_find_decoder(local_codec_par.codec_id).as_ref()
            }.expect("ERROR support codec");
            //
            match  local_codec_par.codec_type {
                ffi::AVMediaType_AVMEDIA_TYPE_VIDEO => {
                    if video_stream_index.is_none() {
                        video_stream_index = Some(i);
                        codec_ptr = local_codec;
                        codec_parament_ptr = local_codec_par;
                    }
                    println!(
                        "Video Codec: resolution {} x {}",
                        local_codec_par.width, local_codec_par.height
                    );
                }
                ffi::AVMediaType_AVMEDIA_TYPE_AUDIO =>{
                    println!(
                        "Audio Codec: {} channels, sample rate {}",
                        local_codec_par.channels, local_codec_par.sample_rate
                    );
                }
                _ => {

                }
            };
            let codec_name = unsafe { CStr::from_ptr(local_codec.name) }
            .to_str()
            .unwrap();

            println!(
                "\tCodec {} ID {} bit_rate {}",
                codec_name, local_codec.id, local_codec_par.bit_rate
            );
    }

    //申请codec上下文内存
    let codec_context = unsafe { ffi::avcodec_alloc_context3(codec_ptr).as_mut()}
        .expect("failed to allocate for codec_context");
    //复制上下文
    if unsafe { ffi::avcodec_parameters_to_context(codec_context, codec_parament_ptr) } < 0 {
        panic!("failed to copy codec paraments to codec context");
    }
    //打开codec
    if unsafe {ffi::avcodec_open2(codec_context, codec_ptr, ptr::null_mut())} < 0 {
        panic!("Failed to open codec");
    }

    //申请一帧内存
    let frame = unsafe { ffi::av_frame_alloc().as_mut() }.expect("Failed to allocate frame buffer");
    //申请包内存
    let packet = unsafe { ffi::av_packet_alloc().as_mut()}.expect("Failed to allocate packet");
    //
    let mut packet_waiting = 8;

    while unsafe { ffi::av_read_frame(format_context, packet)} >= 0 {
        if video_stream_index == Some(packet.stream_index as usize) {
            println!("AVPacket->pts {}", packet.pts);
            //解码视频帧
            // AVHWAccel::
            decode_packet(packet, codec_context, frame).unwrap();
            packet_waiting -= 1;
            if packet_waiting <= 0 {
                break;
            }

        }
         //释放packet
        unsafe { ffi::av_packet_unref(packet) };

    }

    
    println!("releasing all the resources");

    unsafe {
        ffi::avformat_close_input(&mut (format_context as *mut _));
        ffi::av_packet_free(&mut (packet as *mut _));
        ffi::av_frame_free(&mut (frame as *mut _));
        ffi::avcodec_free_context(&mut (codec_context as *mut _));
    }

}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        println!("it works");
    }
}