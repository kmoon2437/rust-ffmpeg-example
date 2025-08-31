mod audio;
mod video;

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    
    match &*args[1] {
        "audio" => audio::audio(args[2].clone()),
        "video" => video::video(args[2].clone()),
        _ => Err(anyhow::anyhow!("audio or video."))
    }
}