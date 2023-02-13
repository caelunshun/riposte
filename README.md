# riposte
A clone of Civilization IV (2005). It is written from scratch (no game engine) and has, at various points in time, consisted of C++ or Lua code. Now, the whole implementation is in Rust. The legacy C++ code is still available in the [`src`](./src) directory.

The graphics rendering uses [`dume`](https://github.com/caelunshun/dume), a 2D renderer I wrote on top of WebGPU, and the UI is based on [`duit`](https://github.com/caelunshun/duit), another one of my libraries.

## Features

* Multiplayer support via networking, proxied through the server in the [`backend`](backend) application
* Procedural map generation using Simplex noise, Poisson sampling, and some other clever tricks
* Basic AI players
* Economy simulation
* Combat simulation
* Cultural influence
* Unit optimal pathfinding
* Many UI components to view the internal economy and production calculations
* Many sound effects from the original game, as well as music dependent on the current era
* Most of the units and technologies of Civ IV through the Medieval Era


## Screenshots
<img width="1440" alt="Screen Shot 2021-11-10 at 4 57 51 PM" src="https://user-images.githubusercontent.com/25177429/218580480-07484d70-2d70-477d-879c-f74f487b90aa.png">
<img width="1440" alt="Screen Shot 2021-11-10 at 4 58 34 PM" src="https://user-images.githubusercontent.com/25177429/218580412-2cc54ccf-2f2b-445c-9d18-b6317cf545e0.png">
<img width="1440" alt="Screen Shot 2021-11-10 at 4 59 18 PM" src="https://user-images.githubusercontent.com/25177429/218580447-bc720541-738f-46fd-96d9-b082392e7a87.png">
