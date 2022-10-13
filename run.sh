
clear 
cargo build

#GNUPGHOME=./gpghome ./target/debug/dcore identity-create --name "Alice" --email "alice@colomba.link"
GNUPGHOME=./gpghome ./target/debug/dcore identity-list-all
# exit 
USER_FINGERPRINT=6DE20CEF9F816A50A5FAD2C6EB30A834AB0ED355
DOC_NAME=test-doc
rm -rf $DOC_NAME
GNUPGHOME=./gpghome ./target/debug/dcore document-create -u $USER_FINGERPRINT -d $DOC_NAME

GNUPGHOME=./gpghome ./target/debug/dcore resource-list-all -u $USER_FINGERPRINT -d $DOC_NAME 

GNUPGHOME=./gpghome ./target/debug/dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config

## next resource-set
GNUPGHOME=./gpghome ./target/debug/dcore resource-set -u $USER_FINGERPRINT \
                                                      -d $DOC_NAME \
                                                      -r config \
                                                      -k hello \
                                                      -v world
GNUPGHOME=./gpghome ./target/debug/dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config
