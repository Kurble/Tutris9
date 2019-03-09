# Tutris 9
This a Tetris99 clone that aims to toy around with matchmaking and online games using rust, web assembly and web sockets. Rendering, audio and input is implemented using quicksilver and networking is handled using stdweb and my own network protocol project mirror.

### Working example
http://tutris.kurble.net

### Setup
As this game is implemented in rust, building is easy. Make sure you have **cargo**, **rustc** and **cargo-web** installed.
You can install **rustc** and **cargo** using [rustup](https://rustup.rs/). To install the cargo-web tool follow [these](https://github.com/koute/cargo-web) instructions.

Running the game on the web requires a simple deploy by running the provided `deploy.sh` script.
The script will build the server and client, and then it will place the server binary and the static files (including client side web assembly) in the deploy folder.

To summarize, run the server like this:
```sh
./deploy.sh
cd deploy
./server --bind-to=127.0.0.1:3000
```

Then open some clients in a browser by going to `localhost:3000`, 
or wherever you can reach the server if you're not running the clients locally.

### License
As Tetris is a trademarked product, 
I would not recommend doing anything else than using this project for educational purposes.
The source code is released under the MIT license.
