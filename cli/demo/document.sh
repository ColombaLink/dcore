GNUPGHOME=./gpghome
DOC_NAME=test-doc
dcore=../../target/debug/cli
USER_FINGERPRINT=50E94443601EF5E11939AD2C6081743EB5F98431

clear
rm -rf $DOC_NAME

echo "create a document"
echo ""

GNUPGHOME=$GNUPGHOME $dcore document-create -u $USER_FINGERPRINT -d $DOC_NAME

echo "list all resources of the document"
echo ""

GNUPGHOME=$GNUPGHOME  $dcore resource-list-all -u $USER_FINGERPRINT -d $DOC_NAME

echo "Cat the content of a specific resource"
echo ""

GNUPGHOME=$GNUPGHOME  $dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config

echo "Mutate the content of a resource"
echo ""

GNUPGHOME=$GNUPGHOME  $dcore resource-set -u $USER_FINGERPRINT \
                                                      -d $DOC_NAME \
                                                      -r config \
                                                      -k ${USER_FINGERPRINT}.remote  \
                                                      -v https://github.com/alice


echo "Cat the mutated resource"
echo ""
GNUPGHOME=$GNUPGHOME  $dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config


### Add resource





rm -rf $DOC_NAME
GNUPGHOME=$GNUPGHOME $dcore document-create -u $USER_FINGERPRINT -d $DOC_NAME
GNUPGHOME=$GNUPGHOME  $dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME --resource-name config
GNUPGHOME=$GNUPGHOME  $dcore resource-set -u $USER_FINGERPRINT \
                                                      -d $DOC_NAME \
                                                      -r config \
                                                      -k ${USER_FINGERPRINT}.remote  \
                                                      -v git@github.com:fuubi/gpgtest.git
GNUPGHOME=$GNUPGHOME  $dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME --resource-name config

#sync
GNUPGHOME=$GNUPGHOME  $dcore document-sync -u $USER_FINGERPRINT -d $DOC_NAME

# new resource
GNUPGHOME=$GNUPGHOME $dcore document-create -u $USER_FINGERPRINT -d $DOC_NAME
GNUPGHOME=$GNUPGHOME  $dcore resource-add -u $USER_FINGERPRINT -d $DOC_NAME --resource-name test
GNUPGHOME=$GNUPGHOME  $dcore resource-list-all -u $USER_FINGERPRINT -d $DOC_NAME
GNUPGHOME=$GNUPGHOME  $dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME --resource-name test

GNUPGHOME=$GNUPGHOME  $dcore resource-set -u $USER_FINGERPRINT \
                                                      -d $DOC_NAME \
                                                      -r test \
                                                      -k hello  \
                                                      -v dcore
GNUPGHOME=$GNUPGHOME  $dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME --resource-name test
