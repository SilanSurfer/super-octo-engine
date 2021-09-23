## Super Octo Engine
Handling of transactions in CSV

## Assumptions

### Error handling

If any error happens while processing transaction then message is printed out with `eprintln!` when output is redirected to file using `>` then error prints should be ignored by OS.


### Transactions

Only transaction **Deposit** creates new client if one doesn't exist. It doesn't make sense in any other case, since:
- **Withdrawal** will take money from the account and new client has balance equal to 0,
- **Dispute**, **Resolve**, **Chargeback** refer transactions that already took place.

If client doesn't exist and one of the transactions, mentioned above, happens transaction is ignored.


### Improvements

- Errors created during processing of transactions could be stored in separate data structure in engine. They could be printed out or log to file with separate CLI flag.
- "Database" stored in `Engine` could be separated into "shards" for example: `database: Vec<Mutex<HashMap<u16, Account>>>`. Then to find account for ClientId:
```
let shard = self.database[hash(key) % db.len()].lock().unwrap()
let account = shard.get(ClientID);
```
This way, in case we have clients with big range of values of ClientIDs` we could minimize problem with obtaining lock on database.