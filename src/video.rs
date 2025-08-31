use std::fs::File;
use std::io::Write;
use std::sync::mpsc;
use ffmpeg_next as ffmpeg;

/// 영상을 디코딩해서 처음 11개 프레임을 raw 이미지 파일로 저장함
/// rgb24 형식에 크기는 원본과 같으니 png 변환은 알아서 따로 하자.
///
/// 변환 명령어 (`1920x1080`을 적절한 크기로 바꿔주면 됨):
/// ```
/// ffmpeg -f rawvideo -s 1920x1080 -pix_fmt rgb24 -i frame.raw image.png
/// ```
pub fn video(src: String) -> anyhow::Result<()> {
    ffmpeg::init()?;
    
    let mut input = ffmpeg::format::input(&src)?;

    let Some(v_stream) = input.streams().best(ffmpeg::media::Type::Video) else {
        anyhow::bail!("No video stream");
    };
    let v_stream_i = v_stream.index();
    let v_context = ffmpeg::codec::context::Context::from_parameters(v_stream.parameters())?;
    let mut v_decoder = v_context.decoder().video()?;
    v_decoder.set_parameters(v_stream.parameters())?;

    let (packet_send, packet_recv) = mpsc::channel();

    std::thread::spawn(move || {
        for (stream, packet) in input.packets() {
            if stream.index() == v_stream_i {
                let _ = packet_send.send(packet);
            }
        }
    });

    let mut v_scaler: Option<ffmpeg::software::scaling::Context> = None;
    let mut i = 0;

    'v_decode: loop {
        let Ok(packet) = packet_recv.recv() else { break };
        v_decoder.send_packet(&packet)?;
        let mut v_frame = ffmpeg::util::frame::Video::empty();

        while v_decoder.receive_frame(&mut v_frame).is_ok() {
            let rebuild_scaler = v_scaler.as_ref().map_or(true, |orig_scaler| {
                return orig_scaler.input().format != v_frame.format()
            });
            if rebuild_scaler {
                v_scaler = Some(
                    ffmpeg::software::scaling::Context::get(
                        v_frame.format(),
                        v_frame.width(), v_frame.height(),
                        ffmpeg::format::Pixel::RGB24,
                        v_frame.width(), v_frame.height(),
                        ffmpeg::software::scaling::Flags::BILINEAR
                    )?
                );
                println!("scaler rebuilt");
                println!("- format: {:?} -> {:?}", v_frame.format(), ffmpeg::format::Pixel::RGB24);
                println!("- v_frame.width = {}", v_frame.width());
                println!("- v_frame.height = {}", v_frame.height());
            }

            // 조금 전에 Some임을 확인했으므로 여기서 unwrap 해도 됨
            let scaler = v_scaler.as_mut().unwrap();

            // rgb 형식으로 변환
            // 이 작업에 gpu를 쓸 수도 있는 걸로 알고 있음
            let mut v_frame_scaled = ffmpeg::util::frame::Video::empty();
            scaler.run(&v_frame, &mut v_frame_scaled)?;

            // ＿人人人人人人人人＿
            // ＞　디코딩 결과물　＜
            // ￣Y^Y^Y^Y^Y^Y^Y^Y^￣
            let decoded_frame = v_frame_scaled.data(0);

            // 얘는 오류가 없었다면 나와야 할 데이터 길이임.
            // rgb 각 1바이트씩일 때 너비 * 높이 * 3 임.
            // 검증 확실하게 할 때 쓰면 됨.
            let _expected_bytes_len = v_frame_scaled.width() * v_frame_scaled.height() * 3;

            // 파일 만들기를 가장한 폴더 더럽히기
            let mut file = File::create(format!("{}_frame{}.raw", src, i))?;
            file.write(&decoded_frame)?;
            i += 1;
            if i > 10 { break 'v_decode; }
        }
    }

    return Ok(());
}