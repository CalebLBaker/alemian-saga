use crate::Platform;

pub enum FileWrapper<P: Platform + ?Sized> {
    Global(P::File),
    User(P::UserFile),
}

impl<P: Platform + ?Sized> std::convert::AsRef<[u8]> for FileWrapper<P> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Global(file) => file.as_ref(),
            Self::User(file) => file.as_ref(),
        }
    }
}
