# o3o debugger

<img width="1701" alt="image" src="https://github.com/user-attachments/assets/508b87f9-a509-4251-a9ff-49a0bf068d90" />
<!-- <img width="1709" alt="image" src="https://github.com/user-attachments/assets/8f2598d3-e7ae-46aa-98ec-b6dc379e2bed" /> -->
<img width="1710" alt="image" src="https://github.com/user-attachments/assets/62a4d920-1d0a-415e-b39b-d09d9cbca045" />
<img width="1710" alt="image" src="https://github.com/user-attachments/assets/94ec9669-7ee0-4a87-b335-82a49cb4a03d" />
<!--<img width="1710" alt="image" src="https://github.com/user-attachments/assets/ef9fea75-5304-42bf-99c9-ae8ed965a53f" />-->
<img width="1710" alt="image" src="https://github.com/user-attachments/assets/4f8b7aca-3cbd-4e08-bd9b-3a43b5159482" />

## Usage

Build the debugger (this will place the executable in `target/release/debugger`):

```
cargo build --release
```

To run the debugger:

```
./dbg <path_to_vcd>
```

This will actually search in multiple places and add the `.vcd` extension if it doesn't exist; for example

```
./dbg cpu
```

will try to find `cpu.vcd` or `build/cpu.vcd`.

To run the debugger locally, replace the above `./debugger` with `cargo run`, e.g.

```
cargo run <path_to_vcd>
```

## Disclaimer

To display individual module screens, the debugger relies on certain signals inside of the
SystemVerilog modules to be present. Examples of these are: `dbg_this_is_cpu` for the `cpu` module, `dbg_this_is_dcache` for the `dcache` module, etc. If these signals are present, the debugger will attempt to render hardcoded data structures of our design, looking for keys defined in the code (`src/headers.rs`). If your design doesn't use the same data structures as ours, however, the debugger will likely panic. **As such, the debugger will likely not work well out of the box for other processors.** It will still show a basic screen and allow you to watch arbitrary signals, but most/all tables will not work.

The debugger is also not very memory-efficient. Performance will likely degrade on `.vcd` files larger than 30MB.

## Contributing

The code here is admittedly not great. However, here is a brief description of the repository structure:

- `src/main.rs` is the entry point of the program, and is the first thing called. Initializes the Ratatui app as well as argument parsing and logging setup.
- `src/app.rs` is where app logic is; the implementations here describe how the main app functions and renders things. This also handles keyboard events.
- `src/snapshots.rs` is where the logic for parsing, storing, and handling queries to the vcd file is. It defines a `Snapshots` struct, which stores objects that hold the values of every variable at every point in time. It also stores an index that keeps track of which snapshot is currently shown, and defines where helper functions like `get_var` get their values from.
- `src/var_index.rs` defines a struct which parses all the variables in a header and stores them in an index object for quick lookup and fuzzy search.
- `src/structures/` defines the various tables for the different data structures we define in the processor (ROB, RS, etc.), and the top-level module defines how to render these/initializes them.

## Cross-Compiling to CAEN

> Disclaimer: This guide is for Apple Silicon Macs (tested on M3 MacBook Air). If you have a different system, you will need to find out how to cross compile to x86 Linux.

To cross-compile to CAEN, you will need to install the rust tool chain for targeting Linux:

```
rustup target add x86_64-unknown-linux-gnu
```

You will also need to install a linker for x86_64:

```
brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu
```

Then, you can run the compilation command:

```
TARGET_CC=x86_64-unknown-linux-gnu cargo build --release --target x86_64-unknown-linux-gnu
```

To transfer to CAEN:

```
scp target/x86_64-unknown-linux-gnu/release/debugger <your uniqname>@login.engin.umich.edu:~/eecs470/p4-w25.group1
```

Remember to change the username and path for your CAEN credentials and desired path!
