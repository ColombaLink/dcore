
clear 
#cargo build

#rm -rf ./gpghome
#mkdir ./gpghome

# exit 
#GNUPGHOME=./gpghome ./target/debug/dcore identity-create --name "Alice" --email "alice@colomba.link"
#GNUPGHOME=./gpghome ./target/debug/dcore identity-list-all
#exit 
USER_FINGERPRINT=FDE9FA99EADEA2DBA5EA3BFF84F61576581A17EE
DOC_NAME=test-doc
rm -rf $DOC_NAME
GNUPGHOME=./gpghome ./target/debug/dcore document-create -u $USER_FINGERPRINT -d $DOC_NAME

GNUPGHOME=./gpghome ./target/debug/dcore resource-list-all -u $USER_FINGERPRINT -d $DOC_NAME 

GNUPGHOME=./gpghome ./target/debug/dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config

## next resource-set
GNUPGHOME=./gpghome ./target/debug/dcore resource-set -u $USER_FINGERPRINT \
                                                      -d $DOC_NAME \
                                                      -r config \
                                                      -k ${USER_FINGERPRINT}.remote  \
                                                      -v https://github.com/alice

GNUPGHOME=./gpghome ./target/debug/dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config
