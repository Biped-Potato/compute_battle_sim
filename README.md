# Realtime Large Scale Battle Simulator with GPU Compute Shaders inside of Bevy
[**Youtube Video and Explanation**]()

![Teaser](img/1million.png)

Description
============
A GPU accelerated battle simulator which heavily utilizes compute shaders to simulate upwards of 2 million units in realtime. The simulation is advanced enough for the units to actually kill each other and can be run at a stable 60 fps with 1 million units on my machine.

Optimizations
=============
The simulation leverages first and foremost parallelism on the GPU with compute shaders to speed up calculations. Additionally, a grid based spatial hashing system also runs across a fixed space to reduce excess calculations between units which are in reality very far away. Units can leverage spatial hashing by iterating through a storage buffer of units which has been sorted based on a unit's position in the grid based spatial hash using a parallel bitonic merge sort algorithm. 
