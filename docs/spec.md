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
56 | 4 | 32-bit little-endian match seed length l
60 | l | match seed
60 + l | 4 | 32-bit little-endian player 1 public seed length m
64 + l | m | player 1 public seed
64 + l + m | 4 | 32-bit little-endian player 2 public seed length n
68 + l + m | n | player 2 public seed
| 68 + l + m + n

### subkey message

offset | size | description
------ | ---- | -----------
0 | 20 | subkey address
| 20
