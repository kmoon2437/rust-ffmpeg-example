use std::fs::File;
use std::io::Write;
use std::sync::mpsc;
use ffmpeg_next as ffmpeg;
use ffmpeg::util::channel_layout::ChannelLayout;

/// 오디오를 디코딩해서 f32 형식의 pcm 파일로 저장함.
/// 엔디언은 시스템 쪽에 맞춰지는 듯.
/// f32le 48000Hz stereo이니 ffplay로 알아서 잘 들어보자.
///
/// 변환 명령어:
/// ```
/// ffmpeg -ar 48000 -f f32le -ch_layout stereo -i audio.pcm -c copy audio.wav
/// ```
///
/// 재생 명령어:
/// ```
/// ffplay -ar 48000 -f f32le -ch_layout stereo -i audio.pcm
/// ```
pub fn audio(src: String) -> anyhow::Result<()> {
    ffmpeg::init()?;

    let mut input = ffmpeg::format::input(&src)?;
    let mut file = File::create(format!("{}.pcm", src))?;

    let Some(a_stream) = input.streams().best(ffmpeg::media::Type::Audio) else {
        anyhow::bail!("No audio stream");
    };
    let a_stream_i = a_stream.index();
    let a_context = ffmpeg::codec::context::Context::from_parameters(a_stream.parameters())?;
    let mut a_decoder = a_context.decoder().audio()?;
    a_decoder.set_parameters(a_stream.parameters())?;

    let (a_packet_sender, a_packet_receiver) = mpsc::channel();

    std::thread::spawn(move || {
        for (stream, packet) in input.packets() {
            if stream.index() == a_stream_i {
                let _ = a_packet_sender.send(packet);
            }
        }
    });

    let mut a_resampler = ffmpeg::software::resampling::Context::get(
        a_decoder.format(),
        a_decoder.channel_layout(),
        a_decoder.rate(),
        ffmpeg::util::format::sample::Sample::F32(
            ffmpeg::util::format::sample::Type::Packed
        ),
        ChannelLayout::STEREO, 48000
    )?;
    println!("converting audio from...");
    println!("- a_decoder.format = {:?}", a_decoder.format());
    println!("- a_decoder.channel_layout = {:?}", a_decoder.channel_layout());
    println!("- a_decoder.rate = {}", a_decoder.rate());

    loop {
        let Ok(packet) = a_packet_receiver.recv() else { break };
        a_decoder.send_packet(&packet)?;
        let mut a_frame = ffmpeg::util::frame::Audio::empty();

        while a_decoder.receive_frame(&mut a_frame).is_ok() {
            let mut a_frame_resampled = ffmpeg::util::frame::Audio::empty();
            a_resampler.run(&a_frame, &mut a_frame_resampled)?;

            // ＿人人人人人人人人＿
            // ＞　디코딩 결과물　＜
            // ￣Y^Y^Y^Y^Y^Y^Y^Y^￣
            let decoded_audio = a_frame_resampled.data(0);

            // 얘는 오류가 없었다면 나와야 할 데이터 길이임.
            // 32비트 (부동소수점) 샘플일 때 샘플레이트 * 채널 수 * 4 임.
            // 검증 확실하게 할 때 쓰면 됨.
            let _expected_bytes_len = a_frame_resampled.samples() * a_frame_resampled.channels() as usize * core::mem::size_of::<f32>();

            // 파일 만들기
            file.write(&decoded_audio)?;
        }
    }

    return Ok(());
}