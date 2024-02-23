#[derive(Debug, Clone)]
pub enum Flow<T, E> {
	Pass,
	Transition(T),
	Failure(E),
}

#[cfg(feature = "from_residual")]
impl<T, E, V> std::ops::FromResidual<Result<T, E>> for Flow<V, E> {
	fn from_residual(residual: Result<T, E>) -> Self {
		if let Err(err) = residual {
			Flow::Failure(err)
		} else {
			unreachable!()
		}
	}
}
