(function() {var implementors = {};
implementors["arcadeum"] = [{"text":"impl&lt;T&gt; Unpin for MerkleTree&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for MerkleProof&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for Tester&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::ID: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Nonce: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Secret: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for JsRng","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for Store&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::ID: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Nonce: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Secret: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for StoreState&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Secret: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for Log&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Event: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for StoreAction&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S, E&gt; Unpin for Context&lt;S, E&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a, S, E&gt; Unpin for MutateSecretInfo&lt;'a, S, E&gt;","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for Proof&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::ID: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Nonce: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for RootProof&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::ID: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Nonce: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for Diff&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for ProofState&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::ID: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Nonce: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for ProofAction&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for PlayerAction&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: Unpin,&nbsp;</span>","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()