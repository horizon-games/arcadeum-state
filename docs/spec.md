# spec

## message

offset | size | description
------ | ---- | -----------
0 | 65 | secp256k1 ecdsa signature of keccak256 hash of bytes 65+
65 | 32 | keccak256 hash of parent message including signature
97 | 4 | 32-bit little-endian message length n
101 | n | message
| 101 + n

### root message

offset | size | description
------ | ---- | -----------
0 | 16 | match id
16 | 20 | player 1 ethereum address
36 | 20 | player 2 ethereum address
| 56

### subkey message

offset | size | description
------ | ---- | -----------
0 | 20 | subkey address
| 20
