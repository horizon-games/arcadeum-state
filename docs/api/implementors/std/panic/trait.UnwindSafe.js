(function() {var implementors = {};
implementors["arcadeum"] = [{"text":"impl&lt;T&gt; UnwindSafe for MerkleTree&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for MerkleProof&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; !UnwindSafe for Tester&lt;S&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for JsRng","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; !UnwindSafe for Store&lt;S&gt;","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; !UnwindSafe for StoreState&lt;S&gt;","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; UnwindSafe for Log&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Event: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; !UnwindSafe for Context&lt;S&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a, S&gt; !UnwindSafe for MutateSecretInfo&lt;'a, S&gt;","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; UnwindSafe for Proof&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::ID: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Nonce: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; UnwindSafe for RootProof&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::ID: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Nonce: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; UnwindSafe for Diff&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; UnwindSafe for ProofState&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::ID: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Nonce: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; UnwindSafe for ProofAction&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; UnwindSafe for PlayerAction&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as State&gt;::Action: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()