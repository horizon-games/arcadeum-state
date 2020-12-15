var searchIndex = JSON.parse('{\
"arcadeum":{"doc":"","i":[[0,"crypto","arcadeum","Cryptographic utilities",null,null],[3,"SecretKey","arcadeum::crypto","Secret key (256-bit) on a secp256k1 curve.",null,null],[6,"Address","","Public key address",null,null],[6,"Signature","","Message signature",null,null],[6,"Hash","","Message digest",null,null],[5,"sign","","Signs a message with a secp256k1 ECDSA secret key.",null,[[["secretkey",3]],["signature",6]]],[5,"recover","","Recovers the address of the key that signed a message.",null,[[],[["address",6],["string",3],["result",4]]]],[5,"_cached_recover","","",null,[[["hash",6],["signature",6]],[["address",6],["string",3],["result",4]]]],[8,"Addressable","","Addressable trait",null,null],[10,"address","","Gets the address.",0,[[],["address",6]]],[11,"eip55","","Gets the EIP 55 representation of the address.",0,[[],["string",3]]],[5,"address","","Computes the address of a secp256k1 ECDSA public key.",null,[[["publickey",3]],["address",6]]],[5,"eip55","","Computes the EIP 55 representation of an address.",null,[[["address",6]],["string",3]]],[5,"keccak256","","Computes the hash specified by the Keccak SHA-3 submission.",null,[[],["hash",6]]],[3,"MerkleTree","","Balanced Merkle tree",null,null],[11,"new","","Constructs an unsalted Merkle tree from a vector.",1,[[["vec",3]]]],[11,"with_salt","","Constructs a salted Merkle tree from a vector and a source…",1,[[["vec",3],["rngcore",8]],[["result",4],["string",3]]]],[11,"deserialize","","Constructs a Merkle tree from its binary representation.",1,[[],[["result",4],["string",3]]]],[11,"serialize","","Generates a binary representation that can be used to…",1,[[],["vec",3]]],[11,"elements","","Gets the elements in the Merkle tree.",1,[[],["vec",3]]],[11,"is_empty","","`true` if the Merkle tree is empty.",1,[[]]],[11,"len","","Gets the number of elements in the Merkle tree.",1,[[]]],[11,"root","","Gets the root hash of the Merkle tree.",1,[[],["hash",6]]],[11,"proof","","Generates a Merkle proof for the element at the given index.",1,[[],[["result",4],["string",3],["merkleproof",3]]]],[3,"MerkleProof","","Merkle proof",null,null],[11,"deserialize","","Constructs a Merkle proof from its binary representation.",2,[[],[["result",4],["string",3]]]],[11,"serialize","","Generates a binary representation that can be used to…",2,[[],["vec",3]]],[11,"element","","Gets the element of the Merkle proof.",2,[[]]],[11,"index","","Gets the index of the element in the Merkle tree.",2,[[]]],[11,"length","","Gets the length of the Merkle tree.",2,[[]]],[11,"root","","Gets the root hash of the Merkle proof.",2,[[],["hash",6]]],[8,"MerkleLeaf","","Merkle tree element trait",null,null],[10,"deserialize","","Constructs an element from its binary representation.",3,[[],[["result",4],["string",3]]]],[10,"serialize","","Generates a binary representation that can be used to…",3,[[],["vec",3]]],[0,"store","arcadeum","Client store",null,null],[0,"bindings","arcadeum::store","WebAssembly-specific utilities",null,null],[3,"Store","","Client [State] store",null,null],[11,"new","","Constructs a new store for a given player.",4,[[["option",4],["player",6]],[["result",4],["string",3]]]],[11,"deserialize","","Constructs a store from its binary representation.",4,[[],[["result",4],["string",3]]]],[11,"serialize","","Generates a binary representation that can be used to…",4,[[["option",4],["player",6]],["vec",3]]],[11,"player","","Gets the player associated with the store.",4,[[],[["option",4],["player",6]]]],[11,"owner","","Gets the author of the store\'s root proof.",4,[[],["address",6]]],[11,"hash","","Gets the hash of the store\'s proof.",4,[[],["hash",6]]],[11,"state","","Gets the state of the store\'s proof.",4,[[],["proofstate",3]]],[11,"pending_player","","Gets the player who must act if in a pending state.",4,[[],[["result",4],["option",4],["string",3]]]],[11,"dispatch_timeout","","Dispatches an action that will continue a stalled…",4,[[],[["result",4],["string",3]]]],[11,"flush","","Dispatches any actions the client is required to send.",4,[[],[["result",4],["string",3]]]],[11,"apply","","Verifies and applies a cryptographically constructed diff…",4,[[["diff",3]],[["result",4],["string",3]]]],[11,"diff","","Generates a diff that can be applied to a store with the…",4,[[["proofaction",3],["vec",3]],[["diff",3],["string",3],["result",4]]]],[3,"StoreState","","Client store state",null,null],[11,"new","","Constructs a new store state.",5,[[]]],[11,"deserialize","","Constructs a state from its binary representation and a…",5,[[],[["result",4],["string",3]]]],[11,"state","","Gets the state of the store state.",5,[[],["option",4]]],[11,"secret","","Gets a player\'s secret state, if available.",5,[[["player",6]],[["box",3],["option",4]]]],[11,"action_count","","Gets the number of actions applied since construction.",5,[[]]],[11,"reveal_count","","Gets the number of secrets revealed since construction.",5,[[]]],[11,"simulate","","Generates an event log resulting from applying an action…",5,[[["option",4],["player",6]],[["result",4],["log",4],["string",3]]]],[4,"Log","","Simulation event log",null,null],[13,"Complete","","A log for a complete transition.",6,null],[13,"Incomplete","","A log for an incomplete transition.",6,null],[3,"StoreAction","","Client store state transition",null,null],[11,"new","","Constructs a new store state transition.",7,[[]]],[8,"State","","Domain-specific store state trait",null,null],[16,"ID","","Identifier type",8,null],[16,"Nonce","","Nonce type",8,null],[16,"Action","","Action type",8,null],[16,"Event","","Event type",8,null],[16,"Secret","","Secret type",8,null],[10,"version","","Gets the ABI version of this implementation.",8,[[]]],[11,"challenge","","Gets the challenge that must be signed in order to certify…",8,[[["address",6]],["string",3]]],[11,"approval","","Gets the approval that must be signed by the owner in…",8,[[["address",6]],["string",3]]],[10,"deserialize","","Constructs a state from its binary representation.",8,[[],[["result",4],["string",3]]]],[11,"is_serializable","","Checks if the state has a binary representation.",8,[[]]],[10,"serialize","","Generates a binary representation that can be used to…",8,[[],[["option",4],["vec",3]]]],[10,"verify","","Verifies if an action by a given player is valid for the…",8,[[["option",4],["player",6]],[["result",4],["string",3]]]],[10,"apply","","Applies an action by a given player to the state.",8,[[["context",3],["option",4],["player",6]],[["box",3],["pin",3]]]],[8,"Secret","","Domain-specific store state secret trait",null,null],[10,"deserialize","","Constructs a state secret from its binary representation.",9,[[],[["result",4],["string",3]]]],[10,"serialize","","Generates a binary representation that can be used to…",9,[[],["vec",3]]],[3,"Context","","[State::apply] utilities",null,null],[11,"mutate_secret","","Mutates a player\'s secret information.",10,[[["player",6]]]],[11,"reveal","","Requests a player\'s secret information.",10,[[["player",6]]]],[11,"reveal_unique","","Requests a player\'s secret information.",10,[[["player",6]]]],[11,"random","","Constructs a random number generator via commit-reveal.",10,[[]]],[11,"log","","Logs an event if logging is enabled.",10,[[]]],[11,"enable_logs","","Enables or disables logging.",10,[[]]],[3,"MutateSecretInfo","","[Context::mutate_secret] callback data",null,null],[12,"secret","","The secret.",11,null],[12,"random","","A source of entropy.",11,null],[12,"log","","An event logger.",11,null],[11,"log","","Logs an event.",11,[[]]],[3,"Tester","","Store tester",null,null],[0,"utils","arcadeum","Utilities",null,null],[5,"hex","arcadeum::utils","Encodes a byte string to its hexadecimal representation.",null,[[],["string",3]]],[5,"unhex","","Decodes the hexadecimal representation of a byte string.",null,[[],[["result",4],["vec",3],["string",3]]]],[0,"version","arcadeum","ABI versioning utilities",null,null],[5,"tag","arcadeum::version","Generates a module exporting a canonical digest of the…",null,[[],["result",6]]],[5,"version","","Generates a canonical digest of the contents of the files…",null,[[],[["vec",3],["result",6]]]],[3,"Proof","arcadeum","Authenticated state",null,null],[11,"new","","Constructs a bare proof from a root proof.",12,[[["rootproof",3]]]],[11,"deserialize","","Updates the proof\'s state from a binary representation.",12,[[],[["result",4],["string",3]]]],[11,"serialize","","Generates a binary representation that can be used to…",12,[[],["vec",3]]],[11,"hash","","Gets the digest of the proof.",12,[[],["hash",6]]],[11,"state","","Gets the state of the proof.",12,[[],["proofstate",3]]],[11,"apply","","Verifies and applies a cryptographically constructed diff…",12,[[["diff",3]],[["error",4],["result",4]]]],[11,"diff","","Generates a diff that can be applied to a proof with the…",12,[[["vec",3],["proofaction",3]],[["diff",3],["result",4],["string",3]]]],[3,"RootProof","","Authenticated initial state",null,null],[11,"new","","Constructs a root proof from `state` and `actions`.",13,[[["vec",3],["proofstate",3],["proofaction",3]],[["result",4],["string",3]]]],[11,"version","","Reads the version from a root proof\'s binary representation.",13,[[],[["result",4],["vec",3],["string",3]]]],[11,"deserialize","","Constructs a root proof from its binary representation.",13,[[],[["result",4],["string",3]]]],[11,"serialize","","Generates a binary representation that can be used to…",13,[[],["vec",3]]],[11,"hash","","Gets the digest of the root proof.",13,[[],["hash",6]]],[11,"author","","Gets the author of the root proof.",13,[[],["address",6]]],[11,"state","","Gets the state of the root proof.",13,[[],["proofstate",3]]],[3,"Diff","","Authenticated state transition",null,null],[11,"deserialize","","Constructs a diff from its binary representation.",14,[[],[["result",4],["string",3]]]],[11,"serialize","","Generates a binary representation that can be used to…",14,[[],["vec",3]]],[11,"proof","","Gets the hash of the proof the diff was constructed on.",14,[[],["hash",6]]],[3,"ProofState","","Consensus state",null,null],[11,"new","","Constructs a consensus state.",15,[[],[["result",4],["string",3]]]],[11,"id","","Gets the identifier of the state.",15,[[]]],[11,"players","","Gets the addresses of the players.",15,[[]]],[11,"player","","Gets the player associated with the given `address`, if…",15,[[["address",6]],[["player",6],["option",4]]]],[11,"state","","Gets the domain-specific state.",15,[[]]],[3,"ProofAction","","Attributable state transition",null,null],[12,"player","","The player performing the action, or [None] if performed…",16,null],[12,"action","","The action.",16,null],[4,"PlayerAction","","State transition",null,null],[13,"Play","","A domain-specific state transition.",17,null],[13,"Certify","","A subkey certification.",17,null],[12,"address","arcadeum::PlayerAction","The subkey address.",18,null],[12,"signature","","The signature of the subkey challenge.",18,null],[13,"Approve","arcadeum","A subkey approval.",17,null],[12,"player","arcadeum::PlayerAction","The player address.",19,null],[12,"subkey","","The subkey address.",19,null],[12,"signature","","The owner\'s signature of the subkey approval.",19,null],[6,"Player","arcadeum","Player identifier",null,null],[8,"State","","Domain-specific state trait",null,null],[16,"ID","","Identifier type",20,null],[16,"Nonce","","Nonce type",20,null],[16,"Action","","Action type",20,null],[10,"version","","Gets the ABI version of this implementation.",20,[[]]],[11,"challenge","","Gets the challenge that must be signed in order to certify…",20,[[["address",6]],["string",3]]],[11,"approval","","Gets the approval that must be signed by the owner in…",20,[[["address",6]],["string",3]]],[10,"deserialize","","Constructs a state from its binary representation.",20,[[],[["result",4],["string",3]]]],[11,"is_serializable","","Checks if the state has a binary representation.",20,[[]]],[10,"serialize","","Generates a binary representation that can be used to…",20,[[],[["option",4],["vec",3]]]],[10,"apply","","Applies an action by a given player to the state.",20,[[["player",6],["option",4]],[["result",4],["string",3]]]],[8,"ID","","Domain-specific identifier trait",null,null],[10,"deserialize","","Consumes an identifier from binary data.",21,[[],[["result",4],["string",3]]]],[10,"serialize","","Generates a binary representation that can be used to…",21,[[],["vec",3]]],[8,"Nonce","","Domain-specific nonce trait",null,null],[10,"deserialize","","Consumes a nonce from binary data.",22,[[],[["result",4],["string",3]]]],[10,"serialize","","Generates a binary representation that can be used to…",22,[[],["vec",3]]],[10,"next","","Gets the next nonce in sequence.",22,[[]]],[8,"Action","","Domain-specific state transition trait",null,null],[10,"deserialize","","Constructs an action from its binary representation.",23,[[],[["result",4],["string",3]]]],[10,"serialize","","Generates a binary representation that can be used to…",23,[[],["vec",3]]],[14,"bind","","Generates WebAssembly bindings for a [super::State].",null,null],[14,"console_log","","`console.log()`",null,null],[11,"from","arcadeum::crypto","",24,[[]]],[11,"into","","",24,[[]]],[11,"to_owned","","",24,[[]]],[11,"clone_into","","",24,[[]]],[11,"borrow","","",24,[[]]],[11,"borrow_mut","","",24,[[]]],[11,"try_from","","",24,[[],["result",4]]],[11,"try_into","","",24,[[],["result",4]]],[11,"type_id","","",24,[[],["typeid",3]]],[11,"vzip","","",24,[[]]],[11,"deserialize","","",1,[[],[["result",4],["string",3]]]],[11,"serialize","","",1,[[],[["global",3],["vec",3]]]],[11,"deserialize","","",1,[[],[["result",4],["string",3]]]],[11,"serialize","","",1,[[],[["global",3],["vec",3]]]],[11,"deserialize","","",1,[[],[["result",4],["string",3]]]],[11,"serialize","","",1,[[],[["global",3],["vec",3]]]],[11,"deserialize","","",1,[[],[["result",4],["string",3]]]],[11,"serialize","","",1,[[],[["global",3],["vec",3]]]],[11,"from","","",1,[[]]],[11,"into","","",1,[[]]],[11,"to_owned","","",1,[[]]],[11,"clone_into","","",1,[[]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"try_into","","",1,[[],["result",4]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"vzip","","",1,[[]]],[11,"deserialize","","",2,[[],[["result",4],["string",3]]]],[11,"serialize","","",2,[[],[["global",3],["vec",3]]]],[11,"deserialize","","",2,[[],[["result",4],["string",3]]]],[11,"serialize","","",2,[[],[["global",3],["vec",3]]]],[11,"deserialize","","",2,[[],[["result",4],["string",3]]]],[11,"serialize","","",2,[[],[["global",3],["vec",3]]]],[11,"deserialize","","",2,[[],[["result",4],["string",3]]]],[11,"serialize","","",2,[[],[["global",3],["vec",3]]]],[11,"from","","",2,[[]]],[11,"into","","",2,[[]]],[11,"to_owned","","",2,[[]]],[11,"clone_into","","",2,[[]]],[11,"borrow","","",2,[[]]],[11,"borrow_mut","","",2,[[]]],[11,"try_from","","",2,[[],["result",4]]],[11,"try_into","","",2,[[],["result",4]]],[11,"type_id","","",2,[[],["typeid",3]]],[11,"vzip","","",2,[[]]],[11,"from","arcadeum::store","",25,[[]]],[11,"into","","",25,[[]]],[11,"borrow","","",25,[[]]],[11,"borrow_mut","","",25,[[]]],[11,"try_from","","",25,[[],["result",4]]],[11,"try_into","","",25,[[],["result",4]]],[11,"type_id","","",25,[[],["typeid",3]]],[11,"vzip","","",25,[[]]],[11,"from","","",4,[[]]],[11,"into","","",4,[[]]],[11,"borrow","","",4,[[]]],[11,"borrow_mut","","",4,[[]]],[11,"try_from","","",4,[[],["result",4]]],[11,"try_into","","",4,[[],["result",4]]],[11,"type_id","","",4,[[],["typeid",3]]],[11,"vzip","","",4,[[]]],[11,"from","","",5,[[]]],[11,"into","","",5,[[]]],[11,"to_owned","","",5,[[]]],[11,"clone_into","","",5,[[]]],[11,"borrow","","",5,[[]]],[11,"borrow_mut","","",5,[[]]],[11,"try_from","","",5,[[],["result",4]]],[11,"try_into","","",5,[[],["result",4]]],[11,"type_id","","",5,[[],["typeid",3]]],[11,"vzip","","",5,[[]]],[11,"from","","",6,[[]]],[11,"into","","",6,[[]]],[11,"borrow","","",6,[[]]],[11,"borrow_mut","","",6,[[]]],[11,"try_from","","",6,[[],["result",4]]],[11,"try_into","","",6,[[],["result",4]]],[11,"type_id","","",6,[[],["typeid",3]]],[11,"vzip","","",6,[[]]],[11,"from","","",7,[[]]],[11,"into","","",7,[[]]],[11,"to_owned","","",7,[[]]],[11,"clone_into","","",7,[[]]],[11,"borrow","","",7,[[]]],[11,"borrow_mut","","",7,[[]]],[11,"try_from","","",7,[[],["result",4]]],[11,"try_into","","",7,[[],["result",4]]],[11,"type_id","","",7,[[],["typeid",3]]],[11,"vzip","","",7,[[]]],[11,"from","","",10,[[]]],[11,"into","","",10,[[]]],[11,"borrow","","",10,[[]]],[11,"borrow_mut","","",10,[[]]],[11,"try_from","","",10,[[],["result",4]]],[11,"try_into","","",10,[[],["result",4]]],[11,"type_id","","",10,[[],["typeid",3]]],[11,"vzip","","",10,[[]]],[11,"from","","",11,[[]]],[11,"into","","",11,[[]]],[11,"borrow","","",11,[[]]],[11,"borrow_mut","","",11,[[]]],[11,"try_from","","",11,[[],["result",4]]],[11,"try_into","","",11,[[],["result",4]]],[11,"type_id","","",11,[[],["typeid",3]]],[11,"vzip","","",11,[[]]],[11,"from","arcadeum","",12,[[]]],[11,"into","","",12,[[]]],[11,"to_owned","","",12,[[]]],[11,"clone_into","","",12,[[]]],[11,"borrow","","",12,[[]]],[11,"borrow_mut","","",12,[[]]],[11,"try_from","","",12,[[],["result",4]]],[11,"try_into","","",12,[[],["result",4]]],[11,"type_id","","",12,[[],["typeid",3]]],[11,"vzip","","",12,[[]]],[11,"from","","",13,[[]]],[11,"into","","",13,[[]]],[11,"to_owned","","",13,[[]]],[11,"clone_into","","",13,[[]]],[11,"borrow","","",13,[[]]],[11,"borrow_mut","","",13,[[]]],[11,"try_from","","",13,[[],["result",4]]],[11,"try_into","","",13,[[],["result",4]]],[11,"type_id","","",13,[[],["typeid",3]]],[11,"vzip","","",13,[[]]],[11,"from","","",14,[[]]],[11,"into","","",14,[[]]],[11,"to_owned","","",14,[[]]],[11,"clone_into","","",14,[[]]],[11,"borrow","","",14,[[]]],[11,"borrow_mut","","",14,[[]]],[11,"try_from","","",14,[[],["result",4]]],[11,"try_into","","",14,[[],["result",4]]],[11,"type_id","","",14,[[],["typeid",3]]],[11,"vzip","","",14,[[]]],[11,"from","","",15,[[]]],[11,"into","","",15,[[]]],[11,"to_owned","","",15,[[]]],[11,"clone_into","","",15,[[]]],[11,"borrow","","",15,[[]]],[11,"borrow_mut","","",15,[[]]],[11,"try_from","","",15,[[],["result",4]]],[11,"try_into","","",15,[[],["result",4]]],[11,"type_id","","",15,[[],["typeid",3]]],[11,"vzip","","",15,[[]]],[11,"from","","",16,[[]]],[11,"into","","",16,[[]]],[11,"to_owned","","",16,[[]]],[11,"clone_into","","",16,[[]]],[11,"borrow","","",16,[[]]],[11,"borrow_mut","","",16,[[]]],[11,"try_from","","",16,[[],["result",4]]],[11,"try_into","","",16,[[],["result",4]]],[11,"type_id","","",16,[[],["typeid",3]]],[11,"vzip","","",16,[[]]],[11,"from","","",17,[[]]],[11,"into","","",17,[[]]],[11,"to_owned","","",17,[[]]],[11,"clone_into","","",17,[[]]],[11,"borrow","","",17,[[]]],[11,"borrow_mut","","",17,[[]]],[11,"try_from","","",17,[[],["result",4]]],[11,"try_into","","",17,[[],["result",4]]],[11,"type_id","","",17,[[],["typeid",3]]],[11,"vzip","","",17,[[]]],[11,"drop","arcadeum::crypto","",24,[[]]],[11,"default","","",24,[[],["secretkey",3]]],[11,"try_from","","",24,[[["scalar",3]],[["secretkey",3],["result",4],["error",4]]]],[11,"fmt","","",24,[[["formatter",3]],[["result",4],["error",3]]]],[11,"fmt","","",24,[[["formatter",3]],[["result",4],["error",3]]]],[11,"eq","","",24,[[["secretkey",3]]]],[11,"ne","","",24,[[["secretkey",3]]]],[11,"clone","","",24,[[],["secretkey",3]]],[11,"into","","",24,[[],["scalar",3]]],[11,"address","","",24,[[],["address",6]]],[11,"address","arcadeum","",26,[[],["address",6]]],[11,"eip55","","",26,[[],["string",3]]],[11,"version","arcadeum::store","",5,[[]]],[11,"challenge","","",5,[[["address",6]],["string",3]]],[11,"approval","","",5,[[["address",6]],["string",3]]],[11,"deserialize","","",5,[[],[["result",4],["string",3]]]],[11,"is_serializable","","",5,[[]]],[11,"serialize","","",5,[[],[["option",4],["vec",3]]]],[11,"apply","","",5,[[["option",4],["player",6]],[["result",4],["string",3]]]],[11,"deserialize","","",7,[[],[["result",4],["string",3]]]],[11,"serialize","","",7,[[],["vec",3]]],[11,"clone","arcadeum::crypto","",1,[[],["merkletree",3]]],[11,"clone","","",2,[[],["merkleproof",3]]],[11,"clone","arcadeum::store","",5,[[],["storestate",3]]],[11,"clone","","",7,[[],["storeaction",3]]],[11,"clone","arcadeum","",12,[[]]],[11,"clone","","",13,[[]]],[11,"clone","","",14,[[],["diff",3]]],[11,"clone","","",15,[[],["proofstate",3]]],[11,"clone","","",16,[[],["proofaction",3]]],[11,"clone","","",17,[[],["playeraction",4]]],[11,"eq","arcadeum::crypto","",1,[[["merkletree",3]]]],[11,"ne","","",1,[[["merkletree",3]]]],[11,"eq","","",2,[[["merkleproof",3]]]],[11,"ne","","",2,[[["merkleproof",3]]]],[11,"deref","arcadeum::store","",11,[[]]],[11,"deref_mut","","",11,[[]]],[11,"fmt","arcadeum::crypto","",1,[[["formatter",3]],["result",6]]],[11,"fmt","","",2,[[["formatter",3]],["result",6]]],[11,"fmt","arcadeum::store","",7,[[["formatter",3]],["result",6]]],[11,"fmt","arcadeum","",14,[[["formatter",3]],["result",6]]],[11,"fmt","","",16,[[["formatter",3]],["result",6]]],[11,"fmt","","",17,[[["formatter",3]],["result",6]]],[11,"serialize","arcadeum::crypto","",1,[[],["result",4]]],[11,"serialize","","",2,[[],["result",4]]],[11,"serialize","arcadeum::store","",6,[[],["result",4]]],[11,"deserialize","arcadeum::crypto","",1,[[],["result",4]]],[11,"deserialize","","",2,[[],["result",4]]],[11,"parse","","",24,[[],[["secretkey",3],["result",4],["error",4]]]],[11,"parse_slice","","",24,[[],[["secretkey",3],["result",4],["error",4]]]],[11,"random","","",24,[[],["secretkey",3]]],[11,"serialize","","",24,[[]]],[11,"tweak_add_assign","","",24,[[["secretkey",3]],[["result",4],["error",4]]]],[11,"tweak_mul_assign","","",24,[[["secretkey",3]],[["result",4],["error",4]]]],[11,"inv","","",24,[[],["secretkey",3]]],[11,"new","arcadeum::store","Constructs a new store tester.",25,[[["vec",3],["proofaction",3]],[["result",4],["string",3]]]],[11,"state","","Gets the state of the tester.",25,[[]]],[11,"secret","","Gets a player\'s secret information.",25,[[["player",6]],[["deref",8],["box",3]]]],[11,"apply","","Applies an action by a given player (or the owner) to the…",25,[[["option",4],["player",6]],[["string",3],["result",4],["vec",3]]]]],"p":[[8,"Addressable"],[3,"MerkleTree"],[3,"MerkleProof"],[8,"MerkleLeaf"],[3,"Store"],[3,"StoreState"],[4,"Log"],[3,"StoreAction"],[8,"State"],[8,"Secret"],[3,"Context"],[3,"MutateSecretInfo"],[3,"Proof"],[3,"RootProof"],[3,"Diff"],[3,"ProofState"],[3,"ProofAction"],[4,"PlayerAction"],[13,"Certify"],[13,"Approve"],[8,"State"],[8,"ID"],[8,"Nonce"],[8,"Action"],[3,"SecretKey"],[3,"Tester"],[6,"Address"]]}\
}');
addSearchOptions(searchIndex);initSearch(searchIndex);