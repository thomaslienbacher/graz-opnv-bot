# Automatisierte Verarbeitung von Verkehrsmeldungen der Graz Holding

- Quelle: https://www.holding-graz.at/de/category/verkehrsmeldungen/


## How to use

Install `matrix-commander-rs` via `$ cargo install matrix-commander`, we will use this 
to send the messages to the default room. This means a chat with the receiving account 
must already exist.
Run the bot at least once to setup the database.
Schedule the `execute-bot.sh` script in the crontab.
