// TraceDraw: backported from eframe 0.37.0, where it sits in
// native/glow_integration.rs with the tests below (see vendor/README.md).
// Kept apart so that TraceDraw's own tests compile and run it
// (`gpu::eframe_context_fallback::tests`): this copy is outside the
// workspace and its own tests run in no gate.

/// Call `create` with `first`, then with each of `fallbacks` until one succeeds.
///
/// Logs every failure, and returns the last error if none succeeds.
pub(crate) fn create_first_context<A: core::fmt::Debug, T, E: core::fmt::Display>(
    first: A,
    fallbacks: impl IntoIterator<Item = A>,
    mut create: impl FnMut(&A) -> core::result::Result<T, E>,
) -> core::result::Result<T, E> {
    let mut result = create(&first);
    let mut attributes = first;
    for fallback in fallbacks {
        let Err(err) = &result else {
            break;
        };
        log::warn!(
            "Failed to create context using context attributes {attributes:?} due to error: {err}"
        );
        log::debug!("Retrying with fallback context attributes: {fallback:?}");
        result = create(&fallback);
        attributes = fallback;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::create_first_context;

    #[test]
    fn create_first_context_stops_at_first_success() {
        let mut tried = vec![];
        let result: Result<&str, String> = create_first_context("core", ["es", "compat"], |&a| {
            tried.push(a);
            if a == "es" {
                Ok(a)
            } else {
                Err(format!("{a} failed"))
            }
        });
        assert_eq!(result, Ok("es"));
        assert_eq!(tried, ["core", "es"]);
    }

    #[test]
    fn create_first_context_tries_every_fallback_and_returns_last_error() {
        let mut tried = vec![];
        let result: Result<(), String> = create_first_context("core", ["es", "compat"], |&a| {
            tried.push(a);
            Err(format!("{a} failed"))
        });
        assert_eq!(result, Err("compat failed".to_owned()));
        assert_eq!(tried, ["core", "es", "compat"]);

        let mut tried = vec![];
        let result: Result<&str, String> = create_first_context("core", ["es", "compat"], |&a| {
            tried.push(a);
            if a == "compat" {
                Ok(a)
            } else {
                Err(format!("{a} failed"))
            }
        });
        assert_eq!(result, Ok("compat"));
        assert_eq!(tried, ["core", "es", "compat"]);
    }
}
