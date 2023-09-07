[![Crates.io](https://img.shields.io/crates/v/comfy.svg)](https://crates.io/crates/comfy)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/darthdeus/comfy#license)
[![Crates.io](https://img.shields.io/crates/d/comfy.svg)](https://crates.io/crates/comfy)
[![Rust](https://github.com/darthdeus/comfy/workflows/CI/badge.svg)](https://github.com/darthdeus/comfy/actions)
[![Discord](https://img.shields.io/discord/691052431525675048.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/M8hySjuG48)

# What is `comfy`?

Comfy is an ergonomic 2D game engine built in Rust. It's designed to be
opinionated, productive, and easy to use. It uses [wgpu](https://wgpu.rs/) and
[winit](https://docs.rs/winit/latest/winit/), which makes it cross-platform,
currently supporting Windows, Linux, MacOS and WASM. Inspired by macroquad,
Raylib, Love2D and many others, it is designed to just work and fill most of
the common use cases.

**Warning**: comfy is currently under heavy development. While there are
games already being made using comfy, the API is not yet stable and
breaking changes will happen. If you want to use comfy for your game you
may be forced to dig into the source code and possibly tweak things
manually. That being said, the source code is designed to be simple and
modifiable.

> `comfy` is named comfy, because it is very comfy to use.

The ultimate goal of comfy is to do the obvious thing as simply as possible
without unnecessray ceremony. If something is annoying to use, it is a bug that
should be fixed. We're not necessarily aiming at beginner friendliness, but
rather productive and ergonomic APIs. If you're a beginner, comfy should be
easy to pick up, but it might not be as polished as some of the other
alternatives. The goal of comfy is ultimately not polish, cleanliness of API,
clean design, type safety, extensibility, or maximum features. It's an engine
that gets out of your way so you can make your game.

# Features

- Simple and productive API.
- Immediate mode rendering for sprites, text and shapes with automatic batching. If you want to draw a circle, you call a function `draw_circle`.
- 2D lighting with HDR, tonemapping and bloom.
- Built-in support for z-index, meaning you don't have to worry about the order of your draw calls.
- [egui](https://egui.rs/) support built in.
- Parallel asset loading with support for most image and audio formats.
- No complex ECS or abstractions to learn. Just build your game and let comfy get out of your way.
- Simple audio using [kira](https://docs.rs/kira/latest/kira/). If you want to play a sound, you call a function `play_sound`.
- Simple 2D camera.
- Text rendering (currently using egui).
- Lots of utilities for common tasks.

# Design goals & philosophy

- Heavy focus on ergonomics and productivity.
- No magic.
- Targeted at 2D games.
- Opinionated and useful defaults.
- **Simple** immediate mode APIs for almost everything.
- Exposed internals for when you need more. Nothing is private.
- Reasonable compile times. Comfy is slower to compile than macroquad, but we
  want to avoid things getting out of hand. End users are not going to be
  required to use any proc macros.
- Global variables are nice. Comfy uses a lot of them.
- Typing less is nice. Comfy has a single context object that gets passed around everywhere.
- Constraints are nice. Comfy wants to be used for a lot of games, but not all of them.

# Non-goals

- 3D support. While it's entirely possible to extend the renderer to
  handle 3D, it is an intentional feature to not even attempt this. There
  is a lot of complexity that comes with 3d models, materials, skeletal
  animations, etc. If you need this complexity, comfy is not for you.
- ECS based engine. While comfy does embed [hecs](https://docs.rs/hecs) and
  provides some helpers for using it, it is by no means required or even
  optimal for most cases.
- Modularity. Comfy is not a modular engine. It's an opinionated toolkit
  with defaults that make sense for most games. There is no intention of
  having a plugin system or the ability to replace wgpu with something
  else.
- Maximum performance. Comfy is not designed to be the fastest engine out
  there. There are many tradeoffs made for the sake of ergonomics and ease of
  use, some of which affect performance. If you're looking for the fastest way
  to draw a million quads, comfy is not for you. If however you have a
  legitimate use case where the performance is not good enough, please open an
  issue. There is a lot of low hanging fruit with respect to performance, but
  as the development is driven by real world usage, unless something shows up
  in a profiler in a game, it's unlikely to be optimized further.

# Getting started

The repository contains many examples under the `comfy/examples` folder.
While there is currently no documentation, the API is simple enough that
just reading the examples should explain things.

# Why use comfy and not X?

## [macroquad](https://macroquad.rs/)

Before I started working on comfy I was using [macroquad](https://macroquad.rs/)
for my games. It works great, but a few things were missing, most notably
RGBA16F textures, which are a feature of OpenGL 3.x, and without which HDR is
not really possible. This is because macroquad targets older versions of GLES
to achieve better cross-platform support. While this is great for many use
cases, at the time I really wanted to play with HDR, bloom and tonemapping,
which lead me down the [wgpu](https://wgpu.rs/) path.

The first version of comfy actually had an API almost identical to macroquad,
where I basically copy pasted function definitions and implemented most of the
functionality on top of wgpu instead. Over time I realized I wanted a few more
things, namely built-in z-index so that my game code wouldn't have to worry
about draw order.

If you like the idea of comfy but it's not stable enough for your use case I
very highly recommend giving macroquad a try. While it is not perfect it has
helped me build a bunch of small games, and most importantly I had fun while
making them.

## [rend3](https://rend3.rs/)

I don't have much experience with rend3 apart from digging a bit through its
code, but as a 3d renderer it fills a very different niche than comfy. If you're
building a 3d game and don't want to do PBR rendering, rend3 is probably
something you want to consider.

## [Fyrox](https://fyrox.rs/)

Fyrox seems like it is trying to fight Unity, Godot and Unreal head on by
currently being the only fully featured Rust game engine, notably also
including a full 3D scene editor. Its 3D demos are very impressive in
particular, and if you're looking for a fully featured 3D engine it's
definitely something to consider.

That being said, comfy is unapologetically focused on 2D games, and as such
fills a very different niche than Fyrox.

## [bevy](https://bevyengine.org/)

Bevy is another contender for the "big Rust game engine" spot. In terms of its
2D features Bevy definitely wins on the size of community and overall crate
support and modularity, but this is something where comfy is not even attempting
to compete. comfy is designed to be opinionated, simple and pragmatic, while
Bevy's goal is to be modular, extensible and build on top of its
all-encompasing ECS.

Due to its modularity Bevy offers many more features through community asset
crates which greatly extend it, but also has a rather distributed and unstable
ecosystem.

Comfy's goal is opposite in many ways. The goal is to provide a simple, stable
and pragmatic foundation. comfy is not a platform for experimenting with Rust's
type system, ECS, or other abstractions. It's a toolkit designed for making
real 2D games.

The only features you'll find in comfy are those which can be immediately used,
understood, and that work from day 1. If a feature is not being used in a real
game it won't appear in the engine source code.

## [ggez](https://ggez.rs/)

ggez is one of those libraries that have been around for a while, but I've
never really got a chance to use it. It does seem to have a bit of a history
with losing maintainers, which is why I never got to use it, as both times when
I was switching frameworks/engines in Rust it was unmaintained. Although in the
current version it did get upgraded to a wgpu-based backend, but I can't speak
to its quality. I would imagine it's a great alternative to macroquad for 2D
games.

---

There are many other frameworks/engines in Rust, but I haven't had a chance to
interact with those in any significant way, hence why they're not in this
comparison.

# Roadmap

The following goals are not in any particular order, but should come reasonably
soon. Comfy is not an aetheral project that will only materialize in 2 years.
Only features that require maximum few weeks of work are listed here.

- Custom shaders/materials.
- Configurable post processing.
- 2D shadowcasters with soft shadows.

# Contributing

Comfy is still very early in its lifecycle. While it has been used to make
games, only a few people have used it or even seen the source code so far.
The best way to contribute is to use comfy and report any issues you find.

The codebase is not clean by any means. It is not the goal of comfy to be the
most beautiful codebase out there. Many things may be suboptimal, and for some
of them it makes a lot of sense to have an open discussion about it. But pull
requests which just reformat the code or move things around or do some kind of
re-organization will likely be rejected unless there was a prior discussion.

If you find anything that does not work as expected please do open an issue.
Comfy is meant to be a productive and ergonomic companion for those who want to
make games.

If something is not ergonomic or you have an idea for how it could be more ergonomic
without sacrificing too much, please open an issue.

If you really just want to make a pull request to contribute _something_
without a prior discussion, the best place are the examples. Both simple and
advanced examples, as well as small example games, are welcome.

Comfy is not currently aiming for heavy documentation coverage due to the rapid
pace of development. Examples are preferred to documentation as they're easier
to fix when APIs change. Most things should be self-explanatory.

# License

comfy is free and open source and dual licensed under MIT and Apache 2.0 licenses.

# TODO:

Examples

- [ ] particles
- [x] text
- [ ] blood canvas
- [ ] raytracing
