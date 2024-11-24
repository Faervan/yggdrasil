# Yggdrasil
- written in the bevy game engine
![2024-10-24_23-06](https://github.com/user-attachments/assets/4744bd46-e67e-4788-a75e-38da0c7547c5)


## Installing
You can either build the game yourself:
### Building from source
```
git clone https://github.com/Faervan/yggdrasil.git
cd yggdrasil
cargo run --release
```
(requires `git`, [rustup](https://rustup.rs/) and some other relatively up-to-date system libraries)

or download a pre-build binary
### Downloading
The download page for binaries is [killarchive.fun/yggdrasil](https://killarchive.fun/yggdrasil)

## Dreams and goals
Inspired by the world of overlord, the dream of this game is to become a (massive?)morpg in a medieval fantasy world...

However, the current goal is to just be a little fun 3D fighting game :)

## Current state
Currently you can shoot little blue bullets and have five lives...multiplayer is working, but only basic client side authority
(all the networking part is handwritten and uses only some basic crates like `tokio`).

## Contributing
I would love to have some other people participate in this :)

Either
- code wise (in rust + bevy)
  - many parts are really messy right know, but I would be willing to explain stuff (or you explain stuff to me) and restructure (which I plan to do anyway)

or
- **art wise**:
  - 3D models (made with e.g. Blender) or
  - sounds/music (like LordLertl does already) or
  - otherwise (e.g. icons, logo and stuff)

 You may also just share some ideas with me, how to improve stuff for example (write me on discord: [faervan](<https://discord.com/users/738658712620630076>))

 ### Report issues and bugs
if you find an issue (which is easier right now than finding none) you are welcome (it is appreciated) to open an
[issue](https://github.com/Faervan/yggdrasil/issues) unless your observed bug is already described in another ticket

## Credits
The model "Undead mage" was generated using [meshy.ai](https://www.meshy.ai/) and is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). The model itself wasn't changed, but animations were applied.
