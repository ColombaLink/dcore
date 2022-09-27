
Algorithm Identifiers for Ed25519, Ed448, X25519, and X448
        for Use in the Internet X.509 Public Key Infrastructure  
https://www.rfc-editor.org/rfc/rfc8410


Using Ed25519 signing keys for encryption

        https://words.filippo.io/using-ed25519-keys-for-encryption/

diffie-hellman (called X25519)



```example work flow 

# Generate a keypair
if(openssl_config()$x25519){
key <- ed25519_keygen()
pubkey <- as.list(key)$pubkey

# Sign message
msg <- serialize(iris, NULL)
sig <- ed25519_sign(msg, key)

# Verify the signature
ed25519_verify(msg, sig, pubkey)

# Diffie Hellman example:
key1 <- x25519_keygen()
key2 <- x25519_keygen()

# Both parties can derive the same secret
x25519_diffie_hellman(key1, key2$pubkey)
x25519_diffie_hellman(key2, key1$pubkey)

# Import/export sodium keys
rawkey <- sodium::sig_keygen()
rawpubkey <- sodium::sig_pubkey(rawkey)
key <- read_ed25519_key(rawkey)
pubkey <- read_ed25519_pubkey(rawpubkey)

# To get the raw key data back for use in sodium
as.list(key)$data
as.list(pubkey)$data

```

- [Just use an existing P2P library, they said](https://spacemesh.io/blog/just-use-an-existing-p2p-library-they-said/)



# Sign git commit 

https://github.com/libgit2/libgit2/blob/ac0f2245510f6c75db1b1e7af7ca01c15dec26bc/tests/libgit2/rebase/sign.c#L89
```c

static int create_cb_signed_gpg(
	git_oid *out,
	const git_signature *author,
	const git_signature *committer,
	const char *message_encoding,
	const char *message,
	const git_tree *tree,
	size_t parent_count,
	const git_commit *parents[],
	void *payload)
{
	git_buf commit_content = GIT_BUF_INIT;
	const char *gpg_signature = "-----BEGIN PGP SIGNATURE-----\n\
\n\
iQIzBAEBCgAdFiEEgVlDEfSlmKn0fvGgK++h5T2/ctIFAlwZcrAACgkQK++h5T2/\n\
ctIPVhAA42RyZhMdKl5Bm0KtQco2scsukIg2y7tjSwhti91zDu3HQgpusjjo0fQx\n\
ZzB+OrmlvQ9CDcGpZ0THIzXD8GRJoDMPqdrvZVrBWkGcHvw7/YPA8skzsjkauJ8W\n\
7lzF5LCuHSS6OUmPT/+5hEHPin5PB3zhfszyC+Q7aujnIuPJMrKiMnUa+w1HWifM\n\
km49OOygQ9S6NQoVuEQede22+c76DlDL7yFghGoo1f0sKCE/9LW6SEnwI/bWv9eo\n\
nom5vOPrvQeJiYCQk+2DyWo8RdSxINtY+G9bPE4RXm+6ZgcXECPm9TYDIWpL36fC\n\
jvtGLs98woWFElOziBMp5Tb630GMcSI+q5ivHfJ3WS5NKLYLHBNK4iSFN0/dgAnB\n\
dj6GcKXKWnIBWn6ZM4o40pcM5KSRUUCLtA0ZmjJH4c4zx3X5fUxd+enwkf3e9VZO\n\
fNKC/+xfq6NfoPUPK9+UnchHpJaJw7RG5tZS+sWCz2xpQ1y3/o49xImNyM3wnpvB\n\
cRAZabqIHpZa9/DIUkELOtCzln6niqkjRgg3M/YCCNznwV+0RNgz87VtyTPerdef\n\
xrqn0+ROMF6ebVqIs6PPtuPkxnAJu7TMKXVB5rFnAewS24e6cIGFzeIA7810py3l\n\
cttVRsdOoego+fiy08eFE+aJIeYiINRGhqOBTsuqG4jIdpdKxPE=\n\
=KbsY\n\
-----END PGP SIGNATURE-----";

	git_repository *repo = (git_repository *)payload;
	int error;

	if ((error = git_commit_create_buffer(&commit_content,
		repo, author, committer, message_encoding, message,
		tree, parent_count, parents)) < 0)
		goto done;

	error = git_commit_create_with_signature(out, repo,
		commit_content.ptr,
		gpg_signature,
		NULL);
		
```



repo.commit_signed()

- [What does a PGP signature on a git commit prove? ](https://people.kernel.org/monsieuricon/what-does-a-pgp-signature-on-a-git-commit-prove)



# Git message format 

Git uses cryptographic signatures in various places, currently objects (tags, commits, mergetags) and transactions (
pushes). In every case, the command which is about to create an object or transaction determines a payload from that,
calls gpg to obtain a detached signature for the payload (gpg -bsa) and embeds the signature into the object or
transaction.

Signatures always begin with -----BEGIN PGP SIGNATURE----- and end with -----END PGP SIGNATURE-----, unless gpg is told
to produce RFC1991 signatures which use MESSAGE instead of SIGNATURE.

Signatures sometimes appear as a part of the normal payload (e.g. a signed tag has the signature block appended after
the payload that the signature applies to), and sometimes appear in the value of an object header (e.g. a merge commit
that merged a signed tag would have the entire tag contents on its "mergetag" header). In the case of the latter, the
usual multi-line formatting rule for object headers applies. I.e. the second and subsequent lines are prefixed with a SP
to signal that the line is continued from the previous line.

https://cdn.kernel.org/pub/software/scm/git/docs/gitformat-signature.html


see here: https://github.com/git/git/blob/36f8e7ed7d72d2ac73743c3c2226cceb29b32156/gpg-interface.c#L35

https://github.com/git/git/blob/9bf691b78cf906751e65d65ba0c6ffdcd9a5a12c/Documentation/gitformat-signature.txt




https://stackoverflow.com/questions/65870508/git-and-sha-256



First, some ruminations on the problems with the current Git signature mechanism.
Ideally, Git would use one of GnuPG's built-in signing mechanisms. If it did so, then it would be easy verify Git
commits without having to invoke Git or to write scripts, by simply using GnuPG's gpg --verify or gpg2 --verify.

# https://stackoverflow.com/questions/23584990/what-data-is-being-signed-when-you-git-commit-gpg-sign-key-id
