# A Boids Demo in Bevy
https://user-images.githubusercontent.com/2727461/190869557-fd8cdb08-434f-42ea-bf1d-f004f9ac17e6.mov

## What is this repo?
 * This is an implementation of the [Boids](https://en.wikipedia.org/wiki/Boids) algorithm in the [Bevy game engine](https://bevyengine.org/).
 * This was a learning project for me as I was new to both Bevy and [Rust](https://www.rust-lang.org/).
 * This is _not_ a terribly clean or directly re-usable implementation of Boids (so sorry), but you're welcome to cherry pick anything you find interesting.

## What is Boids?

[Boids](https://en.wikipedia.org/wiki/Boids) is a simple algorithm for simulating flocking behaviour. It has applications in videogames and nature simulations, and is a common subject for little tech demos such as this one :)

You will find many excellent outlines of the algorithm on the web, so one is not provided here.
 
## Controls

When the demo starts up, you will see some crows flying around somewhat randomly. Over time, they will coalesce into several small flocks or even merge into one large one.

You can adjust several properties of the simulation:
 * *separation* — how strongly the birds try to keep distance from their neighbours
 * *alignment* — how strongly the birds try to keep the same direction as their neighbours
 * *cohesion* — how strongly the birds try to keep close to their neighbours
 * *keep in bounds* — how strongly the birds want to keep from going out of bounds (the rectangular area)
 * *keep level* — how strongly the birds want to avoid up and down movement

Play around with these and give them time to have an impact on the birds' formations.
 
## Try It Out

### Releases

Download a native build under [Releases](https://github.com/HulloImJay/bevy_boids_demo/releases). I have only tested on macOS, but the Windows and Linux builds _should_ work 🤷

### Build It Yerself
If you have Rust installed, you should be able to clone the repo and build and run the demo yourself with `cargo run`.

## Is This How Crows Fly?

Not at all. Although the crow asset used here was inspired by the commonness of the birds in Batticaloa (where I was staying while working on this project), the behaviour here is nothing much beyond a demo of the Boids algorithm and is not a realistic of the behaviour of real crows.

## Assets
There is a house crow model created for this project, which you will find here: _assets/house_crow.blend_. It's a pretty basic, low-poly model with vertex colours and only the crudest animations, but it's also my third model ever so I'm fine with it 😊
![2022-09-16 — House Crow for README(1)](https://user-images.githubusercontent.com/2727461/190869582-d440cf9f-a485-41b8-a646-fe23347cba0e.gif)

## License
Everything in this repo (including the house crow and agouti models under assets) is provided to the public domain and you are free to use or modify as you like without restriction.

## Dinacon 2022
This project was created during [Dinacon 2022](https://www.2022.dinacon.org) in Batticaloa, Sri Lanka. Although it fails to capture the actual behaviour of any wildlife as I had hoped to achieve, I did learn a lot about Rust and Bevy while doing it :)

![Dinacon-2022-Colors-and-Text-and-Logos](https://user-images.githubusercontent.com/2727461/175750323-db1cb815-37c5-4733-81bf-2fb8799334f7.png)

## Why Bevy?

![bevy_logo_light_dark_and_dimmed](https://user-images.githubusercontent.com/2727461/190705825-b2fc723e-ea22-41ec-b5e0-4699fe28b82b.svg)

I've been a Unity developer for more than a decade and decided it's time to learn something new, something using modern software paradigms, something more open and free, and something [without gross ties to the US military](https://kotaku.com/unity-new-contract-us-government-military-army-engine-1849403118). There are a myriad of free and open source game engines being worked on, including some others in Rust, but Bevy seemed like the most promising candidate.
