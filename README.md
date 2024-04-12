## Transaction Engine

Program which read transactions from ".csv" file, process them and print out clients wallet.

### Build
> cargo b

### Usage
Input:
"some_transaction.csv"
```
type, client, tx, amount
deposit, 3, 2, 5.1234
deposit, 6, 3, 5.1234
withdrawal, 6, 9, 4.0
```
Where:  
- type: transaction type (String)
- client: client id (u16)
- tx: transaction id (u32)
- amount: transaction amount (f32) 
How to use:
> cargo run -- some_transaction.csv


Output:
```
client,available,held,total,
3,5.1234,0,5.1234,false
6,1.1234,0,1.1234,false
```
Where:
- client: client id (u16)
- available: client debit (f32) 
- held: client held funds (f32) 
- total: sum of client available and held funds (f32) 

### Structure

The solution is composed of two crates:
- engine - crate responsible for handling transactions and calculating output
- cli - a cli interface for engine crate
