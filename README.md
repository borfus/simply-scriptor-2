# Summary
Simply Scriptor 2 is a general scripting program that records and simulates keyboard and mouse input. Simply Scriptor 2 runs on Windows, Linux, and Mac OS (through a more simplified, command line tool) and is made with Rust.

This project is a continuation of the older [Simply Scriptor](https://github.com/borfus/Simply-Scriptor) project originally made with C.

Simply Scriptor 2 utilizes the [rdev](https://crates.io/crates/rdev) crate to capture and simulate keyboard and mouse events. [GTK](https://crates.io/crates/gtk) is used to provide a simple GUI as well as help provide a few additional features such as file saving/loading.

# Requirements
You can build and run the project using Cargo, Rust's official dependency management and build tool. You will need to have GTK 3 installed on your system to run and build it. There is also a Windows release that includes all of the shared library files for users that don't wish to install GTK 3.

GTK installation documentation can be found on the official GTK website [here](https://www.gtk.org/docs/installations/).

# Usage
When Simply Scriptor 2 is open, you can record a script (shortcut ',' or 'comma'), stop recording a script (shortcut '.' or 'period'), and run the script (shortcut '/' or 'right slash'). If you are using Linux or Windows, you can also click the buttons instead of using keyboard shortcuts if you prefer to do so.

There are a few additional options for recording and running scripts:
- The "Minimize" checkbox automatically minimizes the SS2 window if you click either the "Record" or "Run" buttons.
- "Natural Delay" (checked by default) indicates that the script will mimic the delay that was present while recording the script. This is handy if you need to emulate human-like mouse movement or wait for something to finish in the middle of a script.
    - Unchecking this tells SS2 to run the loaded script as fast as possible and without any delay.
- The "Infinite Loop" checkbox and "Loop Count" number box dictates how many times a script is run.
    - If "Infinite Loop" is unchecked (default), SS2 uses the "Loop Count" value to run the script a certain amount of times (e.g. having the "Infinite Loop" checkbox disabled and a "Loop Count" value of 5 will run the script 5 times before stopping).
    - If "Infinite Loop" is enabled, the "Loop Count" value is disregarded and the script will run forever until it is manually stopped or SS2 is closed.
    - To stop a loop manually, regardless of how many times it will loop, press the '/' or 'right slash' keyboard shortcut to halt the script.

Once you create a script, you have the option of saving it as a `.bin` file. You can also load previously saved script files to prevent the need to record the script each time SS2 is launched.

# Download
You can download the latest version of Simply Scriptor 2 for Linux, Windows, or macOS [here]().

# Current Version
0.2.0

