pub struct StopOnFirstErrorIterator<T, U, E>
where
    T: Iterator<Item = Result<U, E>>,
{
    pub inner: T,
    pub error: Option<E>,
}

impl<T, U, E> StopOnFirstErrorIterator<T, U, E>
where
    T: Iterator<Item = Result<U, E>>,
{
    pub fn new(inner: T) -> Self {
        StopOnFirstErrorIterator { inner, error: None }
    }
}

impl<T, U, E> Iterator for StopOnFirstErrorIterator<T, U, E>
where
    T: Iterator<Item = Result<U, E>>,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(Ok(i)) => Some(i),
            Some(Err(e)) => {
                self.error = Some(e);
                None
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::utils::*;

    #[test]
    fn stop_on_first_error_1() {
        let inner = vec![Ok(1), Ok(2), Err("foo")];
        let mut sut = StopOnFirstErrorIterator::new(inner.into_iter());
        assert_eq!(Some(1), sut.next());
        assert_eq!(Some(2), sut.next());
        assert_eq!(None, sut.next());
        assert!(sut.error.is_some());
        assert_eq!(Some("foo"), sut.error);
    }

    #[test]
    fn stop_on_first_error_2() {
        let inner = vec![Ok(1), Ok(2), Err("foo"), Ok(3), Err("bar")];
        let mut sut = StopOnFirstErrorIterator::new(inner.into_iter());
        assert_eq!(Some(1), sut.next());
        assert_eq!(Some(2), sut.next());
        assert_eq!(None, sut.next());
        let err = sut.error.take();
        assert!(err.is_some());
        assert_eq!(Some("foo"), err);
        assert_eq!(Some(3), sut.next());
        assert_eq!(None, sut.next());
        assert!(sut.error.is_some());
        assert_eq!(Some("bar"), sut.error);
    }
}
