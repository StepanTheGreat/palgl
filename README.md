# PalGL

`wgpu` simplified. Specifically to be easily used in other languages like C.

Using `wgpu` outside of Rust is super difficult (it does have its own bindings, but the descriptor design is super unfriendly for languages like C. 
It's not wgpu's fault of course.)

This crate instead exposes a simplified wgpu context + builder APIs for the common user. It does mean that it will lack any newer or niche features,
since its target is an average graphics developer. Anyone who needs more would use `wgpu` or other backends instead.

One of core features of the library:
- C-API friendly (avoids structs/unions, uses integers and simple values instead)
- Unified context management (Simple context creation)
- GPU Object handles (instead of direct object references, you manipulate simple object handles (ints). FFI friendly and also allows users to retrieve
objects programmatically)
- Bind group cache (no need to manually create cache for those, everything is cleaned up automatically)
- Easier integration with different windowing libraries (abstract handles)