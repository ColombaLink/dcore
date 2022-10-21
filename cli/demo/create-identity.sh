GNUPGHOME=./gpghome
DOC_NAME=test-doc
dcore=../../target/debug/cli
clear

echo "create a new identity"

rm -rf ./gpghome
mkdir ./gpghome

GNUPGHOME=$GNUPGHOME $dcore identity-create --name "Alice" --email "info@colomba.link"
GNUPGHOME=$GNUPGHOME $dcore identity-list-all
