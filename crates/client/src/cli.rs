#[derive(clap::Clap)]
#[clap(setting = clap::AppSettings::ColoredHelp)]
pub(crate) struct Opts {
	#[clap(subcommand)]
	pub subcommand: SubCommand,
}

#[derive(clap::Clap)]
pub(crate) enum SubCommand {
	User(UserSubCommand),
	Blockchain(BlockchainSubCommand),
}

#[derive(clap::Clap)]
pub(crate) enum UserSubCommand {
	Address,
	Balance,
}

#[derive(clap::Clap)]
pub(crate) enum BlockchainSubCommand {
	Len,
	Balance(BlockchainBalanceCommand),
	Transaction(BlockchainTransactionCommand),
}

#[derive(clap::Clap)]
pub(crate) struct BlockchainBalanceCommand {
	pub address: String,
}

#[derive(clap::Clap)]
pub(crate) struct BlockchainTransactionCommand {
	pub address: String,
	pub amount: std::num::NonZeroU64,
}
