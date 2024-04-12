## Transaction Engine

Program which read transactions from ".csv" file, process them and print out clients wallet.

### Build
> cargo b

### Usage
Input:
"some_transaction.csv"
```
type, client, tx, amount
deposit, 3, 1, 5.1234
withdrawal, 3, 2, 4.1
deposit, 6, 3, 5.1234
dispute, 6, 3,
resolve, 6, 3,
dispute, 6, 3,
chargeback, 6, 3,
```
Where:  
- **type:** transaction type (String)
- **client:** client id (u16)
- **tx:** transaction id (u32)
- **amount:** transaction amount (f32)

Transaction types:
- **deposit:** deposit funds (available and total amount increase)
- **withdrawal:** withdrawal funds (available and total amount decrease)
- **dispute:** dispute deposit transaction (if there is an enough funds then funds from disputed transaction are moved from available to held)
- **resolve:** resolve disputed transaction (funds from disputed transaction are moved from held to available)
- **chargeback:** chargeback disputed transaction (held and total funds are decreased by amount from disputed transaction. Account is frozen)

How to use:
> cargo run -- some_transaction.csv


Output:
```
client,available,held,total,locked
3,1.0234003,0,1.0234003,false
6,0,0,0,true
```
Where:
- client: client id (u16)
- available: client debit (f32) 
- held: client held funds (f32) 
- total: sum of client available and held funds (f32) 
- locked: information if chargeback was requested and account is frozen 

### Structure

The solution is composed of two crates:
- engine - crate responsible for handling transactions and calculating output
- cli - a cli interface for engine crate
