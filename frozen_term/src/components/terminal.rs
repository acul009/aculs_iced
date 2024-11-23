pub struct Terminal<B> {
    backend: B,
}

pub trait TerminalBackend {
}