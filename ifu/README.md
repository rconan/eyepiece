# MANIFEST IFU

A simple model for GMT MANIFEST IFU.
There are 3 possible types of IFU to choose from:
 * a 7 hexagons IFU,
 * a round IFU,
 * a slit IFU.

 The model computes the seeing image and write it to `field.png` and for the chosen IFU,
 it masks the image, write it to `<hex|round|slit>_ifu_field.png` and print the IFU throughput.
 
### Usage

The throughput of a circular IFU of diameter 0.5arcsec for r0=20cm , 50degree of zenith at 800nm is found with:
```shell
ifu --r0 20 -z 50 -b I round -d 0.5
```
Replacing the round IFU with a slit the same size:
```shell
ifu --r0 20 -z 50 -b I slit -w 0.5
```
or with the 7 hexagons IFU
```shell
ifu --r0 20 -z 50 -b I hex -w 0.5
```

## Installing 

To use the model, you need first to install [Rust](https://www.rust-lang.org/learn/get-started) and then
install the model with:
```shell
cargo install ifu
```

## Running

The model, with the default 7 hexagon IFU, is run with:
```shell
ifu
```
or run each IFU type with
 * 7 hexagons IFU
 ```shell
ifu hex
```
 * round IFU
 ```shell
ifu round
```
* slit IFU
```shell
ifu slit
```
Check the model options with:
```shell
ifu --help
```
or check each IFU options with
 * 7 hexagons IFU
 ```shell
ifu hex --help
```
 * round IFU
 ```shell
ifu round --help
```
* slit IFU
```shell
ifu slit --help
```

