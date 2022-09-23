gpg --quick-generate-key \
    'Alice B <alice@colomba.link>' \
    ed25519 cert never

gpg --quick-add-key 09681635EA5CA2A31464F57285D47F6836A8ECB3 ed25519 sign 1y

## int the future can just import it
gpg --import ./test/sign-commit/alice@cl.com.gpg


## export
gpg --armor --export 09681635EA5CA2A31464F57285D47F6836A8ECB3
