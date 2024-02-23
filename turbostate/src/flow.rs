#[derive(Debug, Clone)]
pub enum Flow<T, E> {
	Pass,
	Transition(T),
	Failure(E),
}

#[cfg(feature = "from_residual")]
impl<T, E, V, R> std::ops::FromResidual<Result<T, E>> for Flow<V, R>
where
	E: Into<R>,
{
	fn from_residual(residual: Result<T, E>) -> Self {
		if let Err(err) = residual {
			Flow::Failure(err.into())
		} else {
			unreachable!()
		}
	}
}
