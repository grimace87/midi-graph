#[derive(Debug)]
pub enum Error {
    User(String),
    Internal(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Midly(midly::Error),
    Hound(hound::Error),
    Soundfont(soundfont::Error),
    Filter(biquad::Errors),
    CpalBuild(cpal::BuildStreamError),
    CpalPlay(cpal::PlayStreamError),
    NoDevice,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::User(e) => e.fmt(fmt),
            Error::Internal(e) => e.fmt(fmt),
            Error::Io(e) => e.fmt(fmt),
            Error::Json(e) => e.fmt(fmt),
            Error::Midly(e) => e.fmt(fmt),
            Error::Hound(e) => e.fmt(fmt),
            Error::Soundfont(e) => fmt.write_fmt(format_args!("{:?}", e)),
            Error::Filter(e) => fmt.write_fmt(format_args!("{:?}", e)),
            Error::CpalBuild(e) => e.fmt(fmt),
            Error::CpalPlay(e) => e.fmt(fmt),
            Error::NoDevice => "No audio device available".fmt(fmt),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value)
    }
}

impl From<hound::Error> for Error {
    fn from(value: hound::Error) -> Self {
        Error::Hound(value)
    }
}

impl From<midly::Error> for Error {
    fn from(value: midly::Error) -> Self {
        Error::Midly(value)
    }
}

impl From<soundfont::Error> for Error {
    fn from(value: soundfont::Error) -> Self {
        Error::Soundfont(value)
    }
}

impl From<biquad::Errors> for Error {
    fn from(value: biquad::Errors) -> Self {
        Error::Filter(value)
    }
}

impl From<cpal::BuildStreamError> for Error {
    fn from(value: cpal::BuildStreamError) -> Self {
        Error::CpalBuild(value)
    }
}

impl From<cpal::PlayStreamError> for Error {
    fn from(value: cpal::PlayStreamError) -> Self {
        Error::CpalPlay(value)
    }
}
