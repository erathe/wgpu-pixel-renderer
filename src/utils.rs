pub struct Incrementor {
    current: usize,
}

impl Incrementor {
    pub fn new() -> Self {
        Self { current: 0 }
    }
}

impl Iterator for Incrementor {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let num = self.current;
        self.current += 1;
        Some(num)
    }
}
