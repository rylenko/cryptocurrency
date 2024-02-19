<h1 align="center">Welcome to Cryptocurrency 🌿</h1>
Realization of a simple cryptocurrency.

<h1 align="center">Todo</h1>

**-** Replace `impl Into<Cow<'a, str>>` with `impl Into<String>`.

**-** Async.

**-** Tests.

<h1 align="center">Installation</h1>

**1.** Clone this repository how you like it.

**2.** To test the perfomance of the project (So far, only a small part of the project has been covered by tests):
```
$ cargo test --workspace
```

<h1 align="center">Example</h1>

**1.** Create the configuration file **(resources/config.json)**:
```
{
	"nodes": [
		"127.0.0.1:8888",
		"127.0.0.1:9999"
	],
	"package_limits": {
		"max_size": 8192,
		"receive_timeout_secs": 5
	},
	"tracing": {
		"client": {
			"level": "TRACE",
			"path": "resources/client-logs.log"
		},
		"node": {
			"level": "INFO",
			"path": "resources/node-logs.log"
		}
	}
}
```

**2.** Copy the directory for working with two users:
```
$ cp . ../c2
```

**3.** Start the first node **(.)**:
```
$ cargo run node 127.0.0.1:8888
```

**4.** Start the second node **(../c2)**:
```
$ cargo run node 127.0.0.1:9999
```

**5.** Waiting for two nodes to finish genesis block mining.

**6.1** Viewing the balance of the first client **(.)**:
```
$ cargo run client user balance
```

**6.2** We get something like:
```
[127.0.0.1:8888] Balance: 100
[127.0.0.1:9999] Balance: 0
```

**7.1** Viewing the balance of the second client **(../c2)**:
```
$ cargo run client user balance
```

**7.2** We get something like:
```
[127.0.0.1:8888] Balance: 0
[127.0.0.1:9999] Balance: 100
```

This distinction was formed due to the fact that the genesis block is selected by the current user when the miner creates the blockchain. For synchronization, let's perform a couple of transactions to start the mining contest and further synchronization.

**8.1** Viewing the address of the second client **(../c2)**:
```
$ cargo run client user address
```

**8.2** We get something like:
```
13guULBTGwjaZmWmCyy17SsnfGAgomoqNb
```

**9.1** Translating a couple of transactions from the first to the second client **(.)**:
```
$ cargo run client blockchain transaction 13guULBTGwjaZmWmCyy17SsnfGAgomoqNb 10
$ cargo run client blockchain transaction 13guULBTGwjaZmWmCyy17SsnfGAgomoqNb 15
```

**9.2** We get something like:
```
[127.0.0.1:9999] Failed to create transaction: The address does not have enough money.
[127.0.0.1:8888] The transaction was successfully made.

[127.0.0.1:9999] Failed to create transaction: The address does not have enough money.
[127.0.0.1:8888] The transaction was successfully made.
```

After that, the transaction limit per block is reached and mining begins. After successful mining, the first node transfers a new block to the second node. The second node notices that the second block does not fit it and copies the first node's blockchain completely. Synchronization is complete.

**10.1** Let's see what happened to the balance of the first client **(.)**:
```
$ cargo run client user balance
```

**10.2** We get something like:
```
[127.0.0.1:8888] Balance: 74
[127.0.0.1:9999] Balance: 74
```

The balance is not equal to 75, because when the transfer to `STORAGE_REWARD_STARTING_FROM` is exceeded, `STORAGE_REWARD` coins are transferred to the storage at `STORAGE_ADDRESS`. For example, in this case, we got used to the value of 10 coins, so for each transaction on top of one more coin. However, then the balance must be 73. The balance is 74 because we were awarded `MINING_REWARD` coins for mining the block.

**11.1** Let's see what happened to the balance of the second client **(../c2)**:
```
$ cargo run client user balance
```

**11.2** We get something like:
```
[127.0.0.1:8888] Balance: 25
[127.0.0.1:9999] Balance: 25
```

So we made sure that the money that the first client wanted to transfer to the second client had reached.

**12.** To see the other commands:
```
$ cargo run client -h
```
