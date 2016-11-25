% Design Requirements

# Requirements

Function States
- Desired safe
- Unknown (propage?)
- Explicitly safe
- Explicitly unsafe


Annotation storage
- Local attributes/cache
- Crate metadata
- External list (what format, and where is it from?)
 - 
 - An external file would have to be able to encode all function paths

# Draft

- `#[req_safe(type)] fn` - Indicates that safety is desired.
- `#[is_safe(type)] fn` - Indicates that the function is safe (regardless of the body)
- `#[not_safe(type)] fn` - Indicates that the function is NOT safe (regardless of the body)
- `#[tagged_safe(type="db_path",...)] extern crate` - Load safety information for the listed tag from the provided file

