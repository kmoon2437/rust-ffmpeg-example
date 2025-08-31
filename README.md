# rust-ffmpeg-example

<https://github.com/zmwangx/rust-ffmpeg> <-- 이걸 가지고 rust에서 ffmpeg로 영상이랑 오디오 디코딩 해서 써먹는 예시 자료이다.

주석이나 기타 요소 등이 부족하거나 이상하다 싶으면 추가 혹은 수정될 수 있다.

## 빌드 및 실행

우선 <https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building> <-- 여기서 시키는 대로 하자. 원본 글이 영어라 못 읽겠다면 번역기를 쓰면 된다.

다만 윈도우에서 `.cargo/config`에 뭘 추가하라고 하는데 그건 안 해도 되는 거 같다.

그런 다음 이거 받아서 적당히 `cargo run --` 해주면 된다.

인자로 `audio 파일명`을 넣으면 같은 폴더에 `파일명.pcm` 파일이 만들어지고, `video 파일명`을 넣으면 같은 폴더에 `파일명_frame숫자.raw` 파일이 **11개** 정도 만들어진다. **`video` 쪽은 가급적 깨끗한 폴더에서 하자!**

혹시 윈도우에서 뭐가 안 되면 MSYS2를 다시 깔아보자. 필자가 이것 때문에 2시간을 허비했다.