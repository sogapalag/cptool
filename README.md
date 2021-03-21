## Usage
#### Example 1
modify your library name in `main.rs`
```shell
cargo run mod1 ... modn
```
this create curly mod(un-rustfmt version) in `.buffer.rs`

#### Example 2
or if you install some pbcopy, just call
```shell
import.sh mod1 ... modn
```
this just copied rustfmt version into your clipboard, then just `ctrl+V`.

## Warning
Current tool is naive, your args should be without `.rs` and must be leaf module. And it doesn't analysis the module dependencies in library, so you have to add it in args manually.