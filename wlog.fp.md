
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



# something is wrong with the pgp parser 

from the test we can see that certain packages are skipped, 
probably our sig. is also part of it. 

.filter(|(offset, _, _)| {
// skip certain packages we are not (yet) parsing
offset != "1193538" && // invalid mpi
offset != "5053086" && // invalid mpi
offset != "8240010" && // unknown public key algorithm 100
offset != "9758352" && // TODO: unclear why this sig fails to parse
offset != "9797527" && // TODO: unclear why this sig fails to parse
offset != "11855679" && // TODO: unclear why this sig fails to parse
offset != "11855798" && // TODO: unclear why this sig fails to parse
offset != "11856933" && // TODO: unclear why this sig fails to parse
offset != "11857023" && // TODO: unclear why this sig fails to parse
offset != "11857113" && // TODO: unclear why this sig fails to parse
offset != "12688657" && // TODO: unclear why this sig fails to parse
offset != "24798372" && // TODO: unclear why this public sub key fails to parse
offset != "24810682" && // bad attribute size
offset != "38544535" // bad attribute size
});


# Sing the git commit 



```c


int result;
int encoding_is_utf8;
struct strbuf buffer;

assert_oid_type(tree, OBJ_TREE);

if (memchr(msg, '\0', msg_len))
return error("a NUL byte in commit log message not allowed.");

/* Not having i18n.commitencoding is the same as having utf-8 */
encoding_is_utf8 = is_encoding_utf8(git_commit_encoding);

strbuf_init(&buffer, 8192); /* should avoid reallocs for the headers */
strbuf_addf(&buffer, "tree %s\n", oid_to_hex(tree));

/*
 * NOTE! This ordering means that the same exact tree merged with a
 * different order of parents will be a _different_ changeset even
 * if everything else stays the same.
 */
while (parents) {
struct commit *parent = pop_commit(&parents);
strbuf_addf(&buffer, "parent %s\n",
oid_to_hex(&parent->object.oid));
}

/* Person/date information */
if (!author)
author = git_author_info(IDENT_STRICT);
strbuf_addf(&buffer, "author %s\n", author);
if (!committer)
committer = git_committer_info(IDENT_STRICT);
strbuf_addf(&buffer, "committer %s\n", committer);
if (!encoding_is_utf8)
strbuf_addf(&buffer, "encoding %s\n", git_commit_encoding);

while (extra) {
add_extra_header(&buffer, extra);
extra = extra->next;
}
strbuf_addch(&buffer, '\n');

/* And add the comment */
strbuf_add(&buffer, msg, msg_len);

/* And check the encoding */
if (encoding_is_utf8 && !verify_utf8(&buffer))
fprintf(stderr, _(commit_utf8_warn));

if (sign_commit && sign_with_header(&buffer, sign_commit)) {
result = -1;
goto out;
}

result = write_object_file(buffer.buf, buffer.len, OBJ_COMMIT, ret);
out:
strbuf_release(&buffer);


```

# git extra header

https://github.com/git/git/blob/36f8e7ed7d72d2ac73743c3c2226cceb29b32156/commit.c#L1292

```c
struct commit_extra_header *read_commit_extra_headers(struct commit *commit,
const char **exclude)
{
struct commit_extra_header *extra = NULL;
unsigned long size;
const char *buffer = get_commit_buffer(commit, &size);
extra = read_commit_extra_header_lines(buffer, size, exclude);
unuse_commit_buffer(commit, buffer);
return extra;
}
```


# ppg (lib) signatures not working for git 

- problem in both direction
  - so it is probably the library 
- [this example does the same thing as we do](https://blog.hackeriet.no/signing-git-commits-in-rust/) 
  but with a different library. 
- https://github.com/lukaspustina/github-watchtower/blob/master/src/gpg.rs


gpgme: sudo apt install libgpgme-dev



# Rust 

https://stackoverflow.com/questions/41034635/how-do-i-convert-between-string-str-vecu8-and-u8

```c
&str    -> String  | String::from(s) or s.to_string() or s.to_owned()
&str    -> &[u8]   | s.as_bytes()
&str    -> Vec<u8> | s.as_bytes().to_vec() or s.as_bytes().to_owned()
String  -> &str    | &s if possible* else s.as_str()
String  -> &[u8]   | s.as_bytes()
String  -> Vec<u8> | s.into_bytes()
&[u8]   -> &str    | s.to_vec() or s.to_owned()
&[u8]   -> String  | std::str::from_utf8(s).unwrap(), but don't**
&[u8]   -> Vec<u8> | String::from_utf8(s).unwrap(), but don't**
Vec<u8> -> &str    | &s if possible* else s.as_slice()
Vec<u8> -> String  | std::str::from_utf8(&s).unwrap(), but don't**
Vec<u8> -> &[u8]   | String::from_utf8(s).unwrap(), but don't**

* target should have explicit type (i.e., checker can't infer that)

** handle the error properly instead


```
