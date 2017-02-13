use rayon;

pub trait Join: Sized {
    fn join(self, other: Self) -> Self;
}

impl Join for () {
    fn join(self, _: Self) -> Self {
        ()
    }
}

pub trait Split: Sized {
    fn split(self) -> (Self, Self);
}

pub fn join<B, T, F>(n: usize, buffer: B, f: &F) -> T
    where B: Split + Send,
          T: Join + Send,
          F: Fn(B) -> T + Sync
{
    if n == 0 {
        f(buffer)
    }
    else {
        let (left, right) = buffer.split();
        if n == 1 {
            let (left, right) = rayon::join(|| f(left), || f(right));
            left.join(right)
        }
        else {
            let (left, right) = rayon::join(|| join(n - 1, left, f), || join(n - 1, right, f));
            left.join(right)
        }
    }
}
