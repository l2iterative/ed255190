## Implementation for ed25519 in the setting of verifying Tendermint-style EdDSA signatures

<img src="https://github.com/l2iterative/ed255190/blob/main/title.png?raw=true" align="right" width="300">

> The implementation currently present in the repository only handles scalar multiplication of two points, which is one 
> of the bottlenecks and points of interest. It is not a full-fledged implementation for EdDSA needed in Tendermint, 
> which would require format checks and, particularly, SHA-512.

This is a subsequent work of `secp256k10` ([here](https://github.com/l2iterative/secp256k10)) that tries to benchmark the 
performance of ed25519 group arithmetic, which can be used for verifying Tendermint-style signatures. 

There are, however, not a lot of interesting techniques for ed25519. There is no easy endomorphism. 
- If one of the points is known and fixed, the other is not, the best approach appears to be:
  - use a precompiled small lookup table for the fixed point
  - use a small lookup table built in runtime for the other point
  - perform point additions and doublings along the way together

- If both points are known and fixed, the best approach appears to be:
  - use a precompiled large lookup table for each point
  - perform point additions
  - eliminate the necessity of doublings

There could be some other techniques that can improve here and there, but we are not particularly interested in them as 
the gain is probably small. 

We pick affine coordinates for convenience and have the host provide hints to the guest. It is still the case that 
modular multiplication could be more efficient that additions, so Niels coordinates might or might not be a good fit.
We want to emphasize that the gain of this implementation in comparison to a textbook implementation that uses projective 
coordinates and affine Niels coordinates may be small. 

Table loading and unloading can trigger paging, which can be a dominating cost. We provide two ideas on how to reduce it:
- reduce the size of the table by leveraging the negation of points
- increase the segment maximal limit if we expect tables would be reused in one segment if the segment is larger


### Performance

When one of the points is fixed, the other is not, it takes about 303260 cycles to compute `a * G + b * Q` where `G` is
fixed and known, and `Q` does not have a precomputed table with it.

When both points are fixed, the number of cycles have a lot to do with paging. The first access would basically load the 
entire big table of `G` and `Q` in, which causes the first access to take 230156 cycles, while subsequent accesses can be 
done with 115286 cycles. Note that the first access loads two tables, one for `G`, one for `Q`.

We want to add that paging does significantly affect how we want to optimize, and it has a lot to do with whether `Q` is 
being used very often to make it worthwhile to "cache" it by having pre-built tables.

### Tendermint with cache

In the setting of Tendermint, it might make sense to cache the tables for the validators, as long as the validator set 
is not too large. For example, we can assign dedicated memory pages for each validator. In our implementation, each table 
is 60KB (which explains the cycle numbers above, as loading 60KB takes about 60k cycles). If we take 32MB of the 192MB 
addressable memory of the RISC-V VM, it can already hold a cache for 546 validators' public keys.

How useful this caching mechanism can be, however, subject to integration. As we mentioned before, the standard EdDSA signature 
requires SHA512, and RISC Zero only has the SHA256 syscall that could be not used toward SHA512.

### License

The code is dual licensed under MIT or Apache2.0. 