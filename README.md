# Mila Compiler

## !! This is still work in progress and does contain errors. !!

The assignment was to create a compiler for the Mila language.
The code is translated to LLVM IR and then compiled to be able to run the program.

## Dependencies

- Rust (compiled with cargo)
- LLVM version at least 14 - ussually packages llvm & llvm-dev
- clang (tested on version 19.1.7)

For Ubuntu or Debian based OS use:

```bash
sudo apt install llvm llvm-dev clang git
```

To install Rust use:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Running the compiler

To run the program check that you have all dependecies installed. Then simply call the mila bash script: with the following options:

```bash
Usage: $0 [-l] [-t] [-c] [-r] <file> | --clean
  -l   Lex the input file (output tokens)
  -t   Print AST for the input file
  -c   Compile input file to .ir in mtarget/
  -r   Run the program
  -n   Clean the .ir and .s files (used with -c or -r)

  --clean Erase everything in the mtarget directory

You must provide one <file> and at least one of -l, -t, -c[n], -r[n] or clean
```

This script caches the built binaries. So it does not recompile every time you run the program.
To recompile and run use -cr or call mila clean to clean the mtarget directory. To run the programs you
can also run the standart way:

```bash
./mtarget/<name of file w/o ext>
```

## Testing

To test the compiler run `./tests.sh`. This will run all tests from the tests directory and output how many failed or succeded.
