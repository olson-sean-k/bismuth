![Bismuth](https://raw.githubusercontent.com/olson-sean-k/bismuth/master/doc/bismuth.png)

**Bismuth** is a game library that represents a 3D world as an oct-tree that can
be manipulated in real time.

[![Build Status](https://travis-ci.org/olson-sean-k/bismuth.svg?branch=master)](https://travis-ci.org/olson-sean-k/bismuth)
[![Build Status](https://ci.appveyor.com/api/projects/status/1j5kjy2ucps4cpbl/branch/master?svg=true)](https://ci.appveyor.com/project/olson-sean-k/bismuth)
[![Documentation](https://docs.rs/bismuth/badge.svg)](https://docs.rs/bismuth)
[![Crate](https://img.shields.io/crates/v/bismuth.svg)](https://crates.io/crates/bismuth)

## Oct-Tree Structure

An oct-tree is used for spatial partitioning and storing cubic geometry that
represents the game world. Each leaf subdivision is a cube, but its geometry can
be modified by contracting the edges along each axis. This can be used to
approximate curves and other more interesting shapes.

![Screenshot](https://raw.githubusercontent.com/olson-sean-k/bismuth/master/doc/screenshot.png)
