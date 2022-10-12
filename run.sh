clear 
#GNUPGHOME=./gpghome ./target/debug/dcore identity-create --name "Alice" --email "alice@colomba.link"
GNUPGHOME=./gpghome ./target/debug/dcore identity-list-all

USER_FINGERPRINT=148AD79939A9E7DD38B9DA3FA4AB9E2F11404A8F
DOC_NAME=test-doc
rm -rf $DOC_NAME
GNUPGHOME=./gpghome ./target/debug/dcore document-create -u $USER_FINGERPRINT -d $DOC_NAME

GNUPGHOME=./gpghome ./target/debug/dcore resource-list-all -u $USER_FINGERPRINT -d $DOC_NAME 

GNUPGHOME=./gpghome ./target/debug/dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config

## next resource-set
