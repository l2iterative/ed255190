## Efficient implementation for ed25519 in the setting of verifying Tendermint-style EdDSA

> The implementation currently present in the repository only handles scalar multiplication of two points, which is one 
> of the bottlenecks and points of interest. It is not a full-fledged implementation for EdDSA needed in Tendermint, 
> which would require format checks and, particularly, SHA-512.

