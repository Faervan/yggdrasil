# Yggdrasil
- written in the bevy game engine

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
The download page will be [killarchive.fun/yggdrasil](https://killarchive.fun/yggdrasil)

But right now only a Windows executable is available on https://killarchive.fun/yggdrasil.exe

## Dreams and goals
Inspired by the world of overlord, the dream of this game is to become an (massive?)morpg in a medieval fantasy world...

however, the current goal is to just be a little fun 3D fighting game :)

## Current state
Currently you can shoot little blue bullets and have five lives...multiplayer is sometimes working, but not yet reliably
(all the networking part is hand written and uses nothing but the tokio `TcpStream` and `UdpSocket`).

## Contributing
I would love to have some other people participate in this :)

Either
- code wise (in rust + bevy)
  - many parts are really messy right know, but I would be willing to explain stuff (or you explain stuff to me) and restructure (which I plan to do anyways)

or
- **art wise**:
  - 3D models (made with e.g. Blender) or
  - sounds/music (like LordLertl does already) or
  - otherwise (e.g. icons, logo and stuff)

 You may also just share some ideas with me, how to improve stuff for example (write me on discord: [faervan](<https://discord.com/users/738658712620630076>))

 ### Report issues and bugs
if you find an issue (which is easier right know than finding none) you are welcome (it is appreciated) to open an
[issue](https://github.com/Faervan/yggdrasil/issues) if your observed bug is not already described in another ticket
