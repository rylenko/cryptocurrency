use crate::error::SetTracingSubscriberError;

pub fn set_subscriber(
	target: &crate::config::TracingTarget,
) -> Result<
	tracing_appender::non_blocking::WorkerGuard,
	SetTracingSubscriberError,
> {
	use tracing_subscriber::layer::SubscriberExt as _;

	// Create the writer
	let (writer, guard) = tracing_appender::non_blocking(
		tracing_appender::rolling::never("", target.path()),
	);

	// Create and set the subscriber
	let subscriber = tracing_subscriber::Registry::default()
		.with(tracing_subscriber::EnvFilter::new(target.level()))
		.with(tracing_bunyan_formatter::JsonStorageLayer)
		.with(tracing_bunyan_formatter::BunyanFormattingLayer::new(
			String::new(),
			writer,
		));
	tracing::subscriber::set_global_default(subscriber)?;
	Ok(guard)
}
