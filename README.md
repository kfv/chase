# Chase (Chain Sentinel)

Chase is a scalable blockchain monitoring solution designed to provide real-time transaction and activity
tracking across multiple networks. Currently, Chase supports only the Solana network, and is under heavy
development. The primary goal of Chase is to facilitate automatic wallet charging for users of exchange and
trading platforms when transactions are made to their wallets.

## Rationale

Given the potential scale, using pub/sub for a large number of accounts is inefficient (if not impossible.)
Instead, Chase fetches blocks and searches for transactions where the destination (or owner of a token account
being the destination) matches a wallet account. If the transaction is successful and the block is finalised,
Chase triggers an API call to charge the user. This process continues in a loop to ensure all blocks are
and relevant transactions are processed.

## Current Status

Chase is in its early stages, with many features yet to be implemented or refined. The focus has been on
ensuring the correctness of the logic, with plans to optimise the code and expand its functionality over time.
Below is a checklist of upcoming features and required improvements:

- [ ] Support other blockchain networks (requires structural improvements.)
- [ ] Implement authentication for the trigger API.
- [ ] Write tests and add GitHub Actions workflow.
- [ ] Keep track of the last block read, blocks that are fetched but not processed, and the difference between
      the last unprocessed block and the last fetched block to ensure continuity, recoverability, and accuracy
      on restart.
- [ ] Implement a reconnection logic for websocket connections to avoid TCP terminations, ensuring stability even
      though websocket connections can theoretically last indefinitely.
- [ ] Support running the app in two modes: scheduler and worker
  - **Scheduler**: Watches for the arrival of new blocks.
  - **Worker**: Processes blocks, parses transactions, and checks against supported tokens and owned wallets
                to identify relevant transactions.
- [ ] Use Kafka to produce the most recent blocks for processing, where workers will consume these blocks (messages
      shall not be removed until acknowledged by the consumer.) This setup ensures recoverability and decouples the
      workers and schedulers, allowing them to communicate solely through Kafka.
- [ ] Refactor the code to improve overall correctness and maintainability.

## Acknowledgements

The initial development of Chase was sponsored by [OMPFinex](https://www.ompfinex.com).

## Contributing

Discover an issue, see a pathway to enhancement, or have a query? Feel free to open an issue or submit a pull
request. Your engagement is always valued and appreciated!

## License

Distributed under the MIT License. See [LICENSE](LICENSE) for more information.
