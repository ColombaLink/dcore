# Open tasks for Normand

- [ ] 0.1.1: Fix1 - Remove hardcoded user document_utils.rs::43
- [ ] 0.1.1: F2.1 - Fix all test 

Not all test can be run without a user interaction.

From the in the test utils delete the function create_test_env_with_sample_gpg_key(). 

Wherever this function is used we need replace it with the create_test_env_with_test_gpg_key() function.

The difference is that the new function just moves the gpghome folder to the test folder, while the 
previous function created a new directory and tried to import them. This did not work as expected. 

Please see sync_git.rs::sync_with_git() for an example how a test environment should be set up. 

Also, it is important that each test creates an isolated test environment such that the tests can be run in parallel.


