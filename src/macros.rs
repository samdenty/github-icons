#[macro_export]
macro_rules! regex {
  ($re:literal $(,)?) => {{
    static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| regex::Regex::new($re).unwrap())
  }};
}

#[macro_export]
macro_rules! selector {
  ($($selector:expr),+ $(,)?) => {{
    static RE: once_cell::sync::OnceCell<scraper::Selector> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| scraper::Selector::parse(crate::join!(",", $($selector),+)).unwrap())
  }};
}

#[macro_export]
macro_rules! join {
  ($pattern:literal,$first:expr$(, $($rest:expr),*)? $(,)?) => {
    concat!($first$(, $($pattern, $rest),*)?)
  };
}

#[macro_export]
macro_rules! patterns {
  ($($x:expr),+ $(,)?) => (
    [$(glob::Pattern::new($x).unwrap()),+]
  );
}

#[macro_export]
macro_rules! warn_err {
  ($result:expr, $($arg:tt)*) => {{
    if let Err(err) = $result {
      warn!("{} {}", format!($($arg)*), err);
    }
  }};
}
