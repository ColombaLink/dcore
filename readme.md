# Get started

Libp2p depends on the protobuf compiler protoc:

Linux, using apt or apt-get, for example:
```bash
apt install -y protobuf-compiler
protoc --version  # Ensure compiler version is 3+

```

MacOS, using Homebrew:
```bash
brew install protobuf
protoc --version  # Ensure compiler version is 3+
```


# Running tests 

for some tests you need to delete the gpghome directory and kill the agent 

i.e. run the "with_public_key" test function
```bash
rm -rf gpghome/* && killall gpg-agent && cargo test --color=always --package dcore --lib gpg::tests::with_public_key 
```    


