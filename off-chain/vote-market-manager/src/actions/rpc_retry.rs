use retry::delay::Fixed;
use retry::OperationResult;

pub fn retry_rpc<O, R, E, OR>(rpc_operation: O) -> Result<R, retry::Error<E>>
where
    O: FnMut() -> OR,
    OR: Into<OperationResult<R, E>>,
{
    let retry_strategy = Fixed::from_millis(100).take(3);
    retry::retry(retry_strategy, rpc_operation)
}
