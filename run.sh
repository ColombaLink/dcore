GNUPGHOME=./gpghome
USER_FINGERPRINT=7596939E89A540268B6FBA9BB64BDA649FEEC504
DOC_NAME=test-doc

clear 
#cargo build


# rm -rf ./gpghome
# mkdir ./gpghome

# exit 
# GNUPGHOME=$GNUPGHOME ./target/debug/dcore identity-create --name "Alice" --email "alice@colomba.link"
# GNUPGHOME=$GNUPGHOME ./target/debug/dcore identity-list-all

# exit 

rm -rf $DOC_NAME
echo "\n\n"
GNUPGHOME=$GNUPGHOME ./target/debug/dcore document-create -u $USER_FINGERPRINT -d $DOC_NAME

echo "\n\n"
GNUPGHOME=$GNUPGHOME  ./target/debug/dcore resource-list-all -u $USER_FINGERPRINT -d $DOC_NAME 

echo "\n\n"
GNUPGHOME=$GNUPGHOME  ./target/debug/dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config

echo "\n\n"
GNUPGHOME=$GNUPGHOME  ./target/debug/dcore resource-set -u $USER_FINGERPRINT \
                                                      -d $DOC_NAME \
                                                      -r config \
                                                      -k ${USER_FINGERPRINT}.remote  \
                                                      -v https://github.com/alice

echo "\n\n"
GNUPGHOME=$GNUPGHOME  ./target/debug/dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config
